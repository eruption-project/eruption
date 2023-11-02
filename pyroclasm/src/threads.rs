/*  SPDX-License-Identifier: GPL-3.0-or-later  */

/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use std::{sync::atomic::Ordering, thread, time::Duration};

use dbus::blocking::{stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged, Connection};
use flume::{bounded, Sender};
use tracing::{error, info};

use crate::{
    constants,
    dbus_client::{self, profile, slot, Message},
    util, MANAGED_DEVICES,
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

/// Update the tuple of managed devices
pub fn update_managed_devices() -> Result<()> {
    *MANAGED_DEVICES.lock() = dbus_client::get_managed_devices()?;

    Ok(())
}

/// Update the global color map vector
pub fn update_color_map() -> Result<()> {
    let mut led_colors = dbus_client::get_led_colors()?;

    let mut color_map = crate::COLOR_MAP.lock();

    color_map.clear();
    color_map.append(&mut led_colors);

    Ok(())
}

/// Update the slot names
pub fn update_slot_names() -> Result<()> {
    let slot_names = util::get_slot_names()?;

    let mut global_state = crate::STATE.write();
    global_state.slot_names = Some(slot_names);

    Ok(())
}

/// Update the active profile
pub fn update_active_profile() -> Result<()> {
    let active_profile = util::get_active_profile()?;

    let mut global_state = crate::STATE.write();
    global_state.active_profile = Some(active_profile);

    Ok(())
}

/// Update the active slot
pub fn update_active_slot() -> Result<()> {
    let active_slot = util::get_active_slot()?;

    let mut global_state = crate::STATE.write();
    global_state.active_slot = Some(active_slot);

    Ok(())
}

pub fn spawn_events_thread(_events_tx: Sender<dbus_client::Message>) -> Result<()> {
    thread::Builder::new()
        .name("events".to_owned())
        .spawn(move || -> Result<()> {
            // initialize global state
            update_managed_devices()?;

            update_slot_names()?;

            update_active_profile()?;
            update_active_slot()?;

            // spawn D-Bus events thread
            let (dbusevents_tx, dbusevents_rx) = bounded(32);
            spawn_dbus_thread(dbusevents_tx)?;

            // enter the event loop
            'EVENTS_LOOP: loop {
                if crate::QUIT.load(Ordering::SeqCst) {
                    break 'EVENTS_LOOP;
                }

                if let Ok(event) = dbusevents_rx.recv_timeout(Duration::from_millis(0)) {
                    match event {
                        Message::SlotChanged(slot) => {
                            let mut global_state = crate::STATE.write();
                            global_state.active_slot = Some(slot);

                            if let Some(ctx) = &global_state.egui_ctx {
                                ctx.request_repaint();
                            }
                        }

                        Message::SlotNamesChanged(names) => {
                            let mut global_state = crate::STATE.write();
                            global_state.slot_names = Some(names);

                            if let Some(ctx) = &global_state.egui_ctx {
                                ctx.request_repaint();
                            }
                        }

                        Message::ProfileChanged(profile) => {
                            let mut global_state = crate::STATE.write();
                            global_state.active_profile = Some(profile);

                            if let Some(ctx) = &global_state.egui_ctx {
                                ctx.request_repaint();
                            }
                        }

                        Message::BrightnessChanged(brightness) => {
                            let mut global_state = crate::STATE.write();
                            global_state.current_brightness = Some(brightness);

                            if let Some(ctx) = &global_state.egui_ctx {
                                ctx.request_repaint();
                            }
                        }

                        Message::SoundFxChanged(state) => {
                            let mut global_state = crate::STATE.write();
                            global_state.sound_fx = Some(state);

                            if let Some(ctx) = &global_state.egui_ctx {
                                ctx.request_repaint();
                            }
                        }

                        Message::DeviceHotplug(event) => {
                            info!("Device hotplug: {event:?}");

                            update_managed_devices()?;
                        }

                        Message::RulesChanged => {
                            info!("Rules changed");
                        }
                    }
                }

                update_color_map()?;

                thread::sleep(Duration::from_millis(25));
            }

            Ok(())
        })?;

    Ok(())
}

/// Spawn the dbus listener thread
pub fn spawn_dbus_thread(dbus_event_tx: Sender<dbus_client::Message>) -> Result<()> {
    thread::Builder::new()
        .name("dbus".to_owned())
        .spawn(move || -> Result<()> {
            let conn = Connection::new_system().unwrap();

            let slot_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/slot",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let profile_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/profile",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let config_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/config",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let tx = dbus_event_tx.clone();
            let _id1 = slot_proxy.match_signal(
                move |h: slot::OrgEruptionSlotActiveSlotChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    tx.send(Message::SlotChanged(h.slot as usize)).unwrap();

                    true
                },
            )?;

            let tx = dbus_event_tx.clone();
            let _id1_1 = slot_proxy.match_signal(
                move |h: slot::OrgFreedesktopDBusPropertiesPropertiesChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    // slot names have been changed
                    if let Some(args) = h.changed_properties.get("SlotNames") {
                        let slot_names = args
                            .0
                            .as_iter()
                            .unwrap()
                            .map(|v| v.as_str().unwrap().to_string())
                            .collect::<Vec<String>>();
                        tx.send(Message::SlotNamesChanged(slot_names)).unwrap();
                    }

                    true
                },
            )?;

            let tx = dbus_event_tx.clone();
            let _id2 = profile_proxy.match_signal(
                move |h: profile::OrgEruptionProfileActiveProfileChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    let _ = tx
                        .send(Message::ProfileChanged(h.profile_name))
                        .map_err(|e| tracing::error!("Could not send a message: {}", e));

                    true
                },
            )?;

            let tx = dbus_event_tx;
            let _id3 = config_proxy.match_signal(
                move |h: PropertiesPropertiesChanged, _: &Connection, _message: &dbus::Message| {
                    if let Some(brightness) = h.changed_properties.get("Brightness") {
                        let brightness = brightness.0.as_i64().unwrap();

                        tx.send(Message::BrightnessChanged(brightness)).unwrap();
                    }

                    if let Some(result) = h.changed_properties.get("EnableSfx") {
                        let enabled = result.0.as_u64().unwrap() != 0;

                        tx.send(Message::SoundFxChanged(enabled)).unwrap();
                    }

                    true
                },
            )?;

            loop {
                if let Err(e) =
                    conn.process(Duration::from_millis(constants::DBUS_TIMEOUT_MILLIS as u64))
                {
                    error!("Could not process a D-Bus message: {}", e);
                }
            }
        })?;

    Ok(())
}
