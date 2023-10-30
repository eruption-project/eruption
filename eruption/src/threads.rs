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

#[cfg(not(target_os = "windows"))]
use evdev_rs::enums::EV_SYN;
#[cfg(not(target_os = "windows"))]
use evdev_rs::{Device, DeviceWrapper, GrabMode};

use crate::hwdevices::DeviceHandle;
use flume::{bounded, Receiver, Sender};
use palette::{FromColor, Hsva, Lighten, LinSrgba, Saturate, ShiftHue};
use rayon::prelude::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use rayon::slice::ParallelSliceMut;
use std::collections::BTreeMap;

use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::plugins::sdk_support::FRAME_GENERATION_COUNTER_ERUPTION_SDK;
use crate::util::ratelimited;

#[cfg(not(target_os = "windows"))]
use crate::macros;

#[cfg(not(target_os = "windows"))]
use crate::uleds;

use crate::{
    constants, dbus_interface::DbusApi, dbus_interface::Message, hwdevices, plugins, script,
    scripting::parameters::PlainParameter, sdk_support, DeviceAction, EvdevError, MainError,
    COLOR_MAPS_READY_CONDITION, FAILED_TXS, LUA_TXS, QUIT, REQUEST_FAILSAFE_MODE, RGBA,
    ULEDS_SUPPORT_ACTIVE,
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub enum DbusApiEvent {
    ProfilesChanged,
    ActiveProfileChanged,
    ActiveSlotChanged,
    BrightnessChanged,

    HueChanged,
    SaturationChanged,
    LightnessChanged,

    DeviceStatusChanged,
    DeviceHotplug((u16, u16), bool),
}

/// Spawns the D-Bus API thread and executes it's main loop
pub fn spawn_dbus_api_thread(dbus_tx: Sender<Message>) -> plugins::Result<Sender<DbusApiEvent>> {
    let (dbus_api_tx, dbus_api_rx) = bounded(8);

    thread::Builder::new()
        .name("dbus-interface".into())
        .spawn(move || -> Result<()> {
            #[cfg(feature = "profiling")]
            coz::thread_init();

            let dbus = DbusApi::new(dbus_tx)?;

            // will be set to true if we received a dbus event in the current iteration of the loop
            let mut event_received = false;

            loop {
                let timeout = if event_received { 0 } else { 5 };

                // process events, destined for the dbus api
                match dbus_api_rx.recv_timeout(Duration::from_millis(timeout)) {
                    Ok(result) => match result {
                        DbusApiEvent::ProfilesChanged => dbus.notify_profiles_changed()?,

                        DbusApiEvent::ActiveProfileChanged => {
                            dbus.notify_active_profile_changed()?
                        }

                        DbusApiEvent::ActiveSlotChanged => dbus.notify_active_slot_changed()?,

                        DbusApiEvent::BrightnessChanged => dbus.notify_brightness_changed()?,

                        DbusApiEvent::HueChanged => dbus.notify_hue_changed()?,
                        DbusApiEvent::SaturationChanged => dbus.notify_saturation_changed()?,
                        DbusApiEvent::LightnessChanged => dbus.notify_lightness_changed()?,

                        DbusApiEvent::DeviceStatusChanged => dbus.notify_device_status_changed()?,

                        DbusApiEvent::DeviceHotplug(device_info, remove) => {
                            dbus.notify_device_hotplug(device_info, remove)?
                        }
                    },

                    Err(_e) => {
                        event_received = dbus.get_next_event_timeout(0).unwrap_or_else(|e| {
                            error!("Could not get the next D-Bus event: {}", e);

                            false
                        });
                    }
                };
            }
        })?;

    Ok(dbus_api_tx)
}

/// Spawns a device events thread and executes it's main loop
#[cfg(not(target_os = "windows"))]
pub fn spawn_evdev_input_thread(
    tx: Sender<Option<evdev_rs::InputEvent>>,
    handle: DeviceHandle,
    usb_vid: u16,
    usb_pid: u16,
) -> plugins::Result<()> {
    use evdev_rs::enums::{EventCode, EV_REL};
    use tracing::trace;

    use crate::hwdevices::DeviceClass;

    thread::Builder::new()
        .name(format!("events/input:{handle}"))
        .spawn(move || -> Result<()> {
            #[cfg(feature = "profiling")]
            coz::thread_init();

            // wait for Eruption to be started-up completely so that all devices are up and running.
            // Otherwise we would fail the devices before they are even fully initialized
            let mut started = crate::STARTUP_COMPLETED_CONDITION.0.lock();
            while !*started {
                crate::STARTUP_COMPLETED_CONDITION.1.wait(&mut started);
            }

            let evdev_device = match hwdevices::get_input_dev_from_udev(usb_vid, usb_pid) {
                Ok(filename) => match File::open(filename.clone()) {
                    Ok(devfile) => match Device::new_from_file(devfile) {
                        Ok(mut evdev_device) => {
                            debug!("Listening on evdev device: {}", filename);

                            debug!("Device name: '{}'", evdev_device.name().unwrap_or("<n/a>"));

                            debug!(
                                "Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
                                evdev_device.bustype(),
                                evdev_device.vendor_id(),
                                evdev_device.product_id(),
                            );

                            debug!("Driver version: {:x}", evdev_device.driver_version());

                            debug!("Physical location: {}", evdev_device.phys().unwrap_or("<n/a>"));

                            // debug!("Unique identifier: {}", device.uniq().unwrap_or("<n/a>"));

                            debug!("Grabbing the device exclusively");
                            let _ = evdev_device
                                .grab(GrabMode::Grab)
                                .map_err(|e| error!("Could not grab the evdev device exclusively: {}", e));

                            evdev_device
                        }

                        Err(_e) => return Err(EvdevError::EvdevHandleError {}.into()),
                    },

                    Err(_e) => return Err(EvdevError::EvdevError {}.into()),
                }

                Err(_e) => {
                    // device does not have an associated evdev device!?
                    return Err(EvdevError::UdevError {}.into());
                }
            };

            let mut fail = false;

            loop {
                // check if we shall terminate the input thread, before we poll the device
                if fail || QUIT.load(Ordering::SeqCst) {
                    break Ok(());
                }

                if let Some(device) = hwdevices::find_device_by_handle_mut(handle) {
                    if device.read().has_failed()? {
                        warn!("Terminating evdev input thread due to a failed device");

                        // we need to terminate and then re-enter the main loop to update all global state
                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                        break Ok(());
                    }

                    let device_class = device.read().get_device_class();
                    match device_class {
                        DeviceClass::Keyboard => {
                            match evdev_device.next_event(
                                evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING,
                            ) {
                                Ok(k) => {
                                    trace!("Key event: {:?}", k.1);

                                    // reset "to be dropped" flag
                                    macros::DROP_CURRENT_KEY.store(false, Ordering::SeqCst);

                                    // update our internal representation of the keyboard state
                                    if let EventCode::EV_KEY(ref code) = k.1.event_code {
                                        let is_pressed = k.1.value > 0;
                                        let index =
                                            device.read().as_keyboard_device().unwrap().ev_key_to_key_index(*code) as usize;

                                        if let Some(mut v) =
                                            crate::KEY_STATES.write().get_mut(index)
                                        {
                                            *v = is_pressed;
                                        } else {
                                            ratelimited::error!("Could not update key states");
                                        }
                                    }

                                    tx.send(Some(k.1)).unwrap_or_else(|e| {
                                        ratelimited::error!(
                                            "Could not send a keyboard event to the main thread: {}",
                                            e
                                        );

                                        // mark the device as failed
                                       device
                                            .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                            .and_then(|mut device| {
                                        device.fail().map_err(|e| {
                                                    ratelimited::error!("An error occurred while trying to mark the device as failed: {e}")
                                                })
                                                .ok()
                                            });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        fail = true;
                                    });

                                    // update AFK timer
                                    *crate::LAST_INPUT_TIME.write() = Instant::now();
                                }

                                Err(e) => {
                                    if e.raw_os_error().unwrap() == libc::ENODEV {
                                        warn!("Keyboard device disappeared: {}", e);

                                        device
                                        .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                        .and_then(|mut device| {
                                    device.close_all().map_err(|e| {
                                                ratelimited::error!("An error occurred while closing the device: {e}")
                                            })
                                            .ok()
                                        });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    } else {
                                        error!("Could not peek an evdev event: {}", e);

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    }
                                }
                            }
                        }

                        DeviceClass::Mouse => {
                            match evdev_device.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
                                Ok(k) => {
                                    // trace!("Mouse event: {:?}", k.1);

                                    // reset "to be dropped" flag
                                    macros::DROP_CURRENT_MOUSE_INPUT.store(false, Ordering::SeqCst);

                                    // update our internal representation of the device state
                                    if let EventCode::EV_SYN(code) = k.1.clone().event_code {
                                        if code == EV_SYN::SYN_DROPPED {
                                            warn!("Device {handle} dropped some events, resyncing...");
                                            evdev_device.next_event(evdev_rs::ReadFlag::SYNC)?;
                                        } else {
                                            // directly mirror SYN events to reduce input lag
                                  macros::UINPUT_TX
                                                .read()
                                                .as_ref()
                                                .unwrap()
                                                .send(macros::Message::MirrorMouseEventImmediate(
                                                    k.1.clone(),
                                                ))
                                                .unwrap_or_else(|e| {
                                                    ratelimited::error!(
                                                                "Could not send a pending mouse event: {}",
                                                                e
                                                            )
                                                });
                                        }
                                    } else if let EventCode::EV_KEY(code) =
                                        k.1.clone().event_code
                                    {
                                        let is_pressed = k.1.value > 0;
                                        match device.read().as_mouse_device().unwrap().ev_key_to_button_index(code) {
                                            Ok(index) => {
                                                if let Some(mut v) =
                                                    crate::BUTTON_STATES.write().get_mut(index as usize)
                                                {
                                                    *v = is_pressed;
                                                } else {
                                                    ratelimited::error!("Could not update mouse-button states");
                                                }
                                            }

                                            Err(e) => {
                                                tracing::warn!("Mouse event for '{code:?}' not processed: {e}")
                                            }
                                        }
                                    } else if let EventCode::EV_REL(code) =
                                        k.1.clone().event_code
                                    {
                                        // ignore mouse wheel related events here
                                        if code != EV_REL::REL_WHEEL
                                            && code != EV_REL::REL_HWHEEL
                                            && code != EV_REL::REL_WHEEL_HI_RES
                                            && code != EV_REL::REL_HWHEEL_HI_RES
                                        {
                                            // immediately mirror pointer motion events to reduce input-lag.
                                            // This currently prohibits further manipulation of pointer motion events
                                  macros::UINPUT_TX
                                                .read()
                                                .as_ref()
                                                .unwrap()
                                                .send(macros::Message::MirrorMouseEventImmediate(
                                                    k.1.clone(),
                                                ))
                                                .unwrap_or_else(|e| {
                                                    ratelimited::error!(
                                                                "Could not send a pending mouse event: {}",
                                                                e
                                                            )
                                                });
                                        }
                                    }

                                    tx.send(Some(k.1)).unwrap_or_else(|e| {
                                        ratelimited::error!(
                                                    "Could not send a mouse event to the main thread: {}",
                                                    e
                                                );

                                        // mark the device as failed
                                       device
                                            .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                            .and_then(|mut device| {
                                        device.fail().map_err(|e| {
                                                    ratelimited::error!("An error occurred while trying to mark the device as failed: {e}")
                                                })
                                                .ok()
                                            });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        fail = true;
                                    });

                                    // update AFK timer
                                    *crate::LAST_INPUT_TIME.write() = Instant::now();
                                }

                                Err(e) => {
                                    if e.raw_os_error().unwrap() == libc::ENODEV {
                                        warn!("Mouse device disappeared: {}", e);

                                     device
                                        .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                        .and_then(|mut device| {
                                    device.close_all().map_err(|e| {
                                                ratelimited::error!("An error occurred while closing the device: {e}")
                                            })
                                            .ok()
                                        });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    } else {
                                        error!("Could not peek an evdev event: {}", e);

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    }
                                }
                            }
                        }

                        DeviceClass::Misc => {
                            match evdev_device.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
                                Ok(k) => {
                                    trace!("Misc input event: {:?}", k.1);

                                    // reset "to be dropped" flag
                                    // macros::DROP_CURRENT_MISC_INPUT.store(false, Ordering::SeqCst);

                                    // directly mirror pointer motion events to reduce input lag.
                                    // This currently prohibits further manipulation of pointer motion events
                                    macros::UINPUT_TX
                                        .read()
                                        .as_ref()
                                        .unwrap()
                                        .send(macros::Message::MirrorKey(k.1.clone()))
                                        .unwrap_or_else(|e| {
                                            ratelimited::error!(
                                                    "Could not send a pending misc device input event: {}",
                                                    e
                                                )
                                        });

                                    tx.send(Some(k.1)).unwrap_or_else(|e| {
                                        ratelimited::error!(
                                                "Could not send a misc device input event to the main thread: {}",
                                                e
                                            );

                                        // mark the device as failed
                                       device
                                            .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                            .and_then(|mut device| {
                                        device.fail().map_err(|e| {
                                                    ratelimited::error!("An error occurred while trying to mark the device as failed: {e}")
                                                })
                                                .ok()
                                            });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        fail = true;
                                    });

                                    // update AFK timer
                                    *crate::LAST_INPUT_TIME.write() = Instant::now();
                                }

                                Err(e) => {
                                    if e.raw_os_error().unwrap() == libc::ENODEV {
                                        warn!("Misc device disappeared: {}", e);

                                     device
                                        .try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS)
                                        .and_then(|mut device| {
                                    device.close_all().map_err(|e| {
                                                ratelimited::error!("An error occurred while closing the device: {e}")
                                            })
                                            .ok()
                                        });

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    } else {
                                        error!("Could not peek evdev event: {}", e);

                                        // we need to terminate and then re-enter the main loop to update all global state
                                        crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                        return Err(EvdevError::EvdevEventError {}.into());
                                    }
                                }
                            }
                        }

                        _ => {
                            tracing::error!("Invalid device class");
                        }
                    }
            } else {
                ratelimited::error!("Could not find the device using its handle");
            }
        }
    })?;

    Ok(())
}

pub fn spawn_lua_thread(
    thread_idx: usize,
    lua_rx: Receiver<script::Message>,
    script_file: &Path,
    parameters: &[PlainParameter],
) -> Result<()> {
    info!("Loading Lua script: {}", script_file.display());

    let builder = thread::Builder::new()
        // .stack_size(4096 * 32)
        .name(format!(
            "{}:{}",
            thread_idx,
            script_file.file_name().unwrap().to_string_lossy(),
        ));

    let script_file = script_file.to_path_buf();
    let mut parameter_values: BTreeMap<String, PlainParameter> = parameters
        .iter()
        .map(|pv| (pv.name.clone(), pv.clone()))
        .collect();

    builder.spawn(move || -> Result<()> {
        #[cfg(feature = "profiling")]
        coz::thread_init();

        loop {
            let result = script::run_script(&script_file, &mut parameter_values, &lua_rx);

            match result {
                Ok(script::RunScriptResult::RestartScript) => {
                    debug!("Restarting script {}", script_file.to_string_lossy());
                }

                Ok(script::RunScriptResult::TerminatedGracefully) => return Ok(()),

                Ok(script::RunScriptResult::TerminatedWithErrors) => {
                    error!("Script execution failed");

                    if let Some(tx) = LUA_TXS.write().get_mut(thread_idx) {
                        tx.is_failed = true;
                    }

                    REQUEST_FAILSAFE_MODE.store(true, Ordering::SeqCst);

                    return Err(MainError::ScriptExecError {}.into());
                }

                Err(_e) => {
                    error!("Script execution failed due to an unknown error");

                    if let Some(tx) = LUA_TXS.write().get_mut(thread_idx) {
                        tx.is_failed = true;
                    }

                    REQUEST_FAILSAFE_MODE.store(true, Ordering::SeqCst);

                    return Err(MainError::ScriptExecError {}.into());
                }
            }
        }
    })?;

    Ok(())
}

pub fn spawn_device_io_thread(dev_io_rx: Receiver<DeviceAction>) -> Result<()> {
    let builder = thread::Builder::new().name("dev-io/all".to_owned());
    builder.spawn(move || -> Result<()> {
        #[cfg(feature = "profiling")]
        coz::thread_init();

        // stores the generation number of the frame that is currently visible on the keyboard
        let saved_frame_generation = AtomicUsize::new(0);
        let saved_frame_generation_eruption_sdk = AtomicUsize::new(0);

        // used to calculate frames per second
        let mut fps_counter: i32 = 0;
        let mut fps_timer = Instant::now();

        #[allow(clippy::never_loop)]
        loop {
            // check if we shall terminate the device I/O thread
            if QUIT.load(Ordering::SeqCst) {
                break Ok(());
            }

            match dev_io_rx.recv() {
                Ok(message) => match message {
                    DeviceAction::RenderNow  => {
                        // If we are in the process of switching between profiles, we need to postpone rendering
                        // until the switch has been completed, to avoid getting into inconsistent states
                        let switching_completed = crate::PROFILE_SWITCHING_COMPLETED_CONDITION.0.lock();
                        let current_frame_generation = script::FRAME_GENERATION_COUNTER.load(Ordering::SeqCst);

                        if *switching_completed && saved_frame_generation.load(Ordering::SeqCst) < current_frame_generation {
                            // instruct the Lua VMs to realize their color maps, but only if at least one VM
                            // submitted a new color map (performed a frame generation increment)

                            // execute render "pipeline" now...
                            let mut drop_frame = false;

                            // // first, start with a clear canvas
                            // script::LED_MAP.write().copy_from_slice(
                            //     &[RGBA {
                            //         r: 0,
                            //         g: 0,
                            //         b: 0,
                            //         a: 0,
                            //     }; constants::CANVAS_SIZE],
                            // );

                            // instruct Lua VMs to realize their color maps,
                            // (to blend their local color maps with the canvas)
                            *COLOR_MAPS_READY_CONDITION.0.lock() = LUA_TXS.read().len().saturating_sub(FAILED_TXS.read().len());

                            for (index, lua_tx) in LUA_TXS.read().iter().enumerate() {
                                // if this tx failed previously, then skip it completely
                                if !FAILED_TXS.read().contains(&index) {
                                    // guarantee the right order of execution for the alpha blend
                                    // operations, so we have to wait for the current Lua VM to
                                    // complete its blending code, before continuing
                                    let mut pending = COLOR_MAPS_READY_CONDITION.0.lock();

                                    let mut errors_present = false;

                                    lua_tx
                                        .send(script::Message::RealizeColorMap)
                                        .unwrap_or_else(|e| {
                                            errors_present = true;

                                            // this will happen most likely during switching of profiles
                                            ratelimited::debug!("Send error during realization of color maps: {}", e);
                                            FAILED_TXS.write().insert(index);
                                        });


                                    if errors_present {
                                        drop_frame = true;
                                        ratelimited::debug!("Frame dropped: Error while waiting for the color map");
                                        break;
                                    }

                                    let result = COLOR_MAPS_READY_CONDITION.1.wait_for(
                                        &mut pending,
                                        Duration::from_millis(constants::TIMEOUT_REALIZE_COLOR_MAP_CONDITION_MILLIS),
                                    );

                                    if result.timed_out() {
                                        ratelimited::debug!("At least one script skipped submitting an updated color map");
                                        break;
                                    }
                                } else {
                                    drop_frame = true;
                                    break;
                                }
                            }

                            #[inline]
                            fn ease_in_out_quad(x: f32) -> f32 {
                                if x < 0.5 {
                                    2.0 * x * x
                                } else {
                                    1.0 - f32::powf(-2.0 * x + 2.0, 2.0) / 2.0
                                }
                            }

                            // alpha blend the color maps of the last active profile with the current canvas
                            let fader_base = crate::FADER_BASE.load(Ordering::SeqCst);
                            if fader_base > 0 {
                                let fader = crate::FADER.load(Ordering::SeqCst);

                                // let alpha = fader_base as f32 / fader as f32;
                                let alpha = ease_in_out_quad(fader_base as f32 / fader as f32);

                                if alpha > 0.009 {
                                    let saved_led_map = script::SAVED_LED_MAP.read();

                                    for canvas in script::LED_MAP.write().chunks_exact_mut(constants::CANVAS_SIZE) {
                                        alpha_blend(&saved_led_map, canvas,alpha);
                                    }
                                } else {
                                    crate::FADER_BASE.store(0, Ordering::SeqCst);
                                }
                            }

                            // finally, blend the LED map of the SDK support plugin
                            let current_frame_generation_eruption_sdk = FRAME_GENERATION_COUNTER_ERUPTION_SDK.load(Ordering::SeqCst);
                            if saved_frame_generation_eruption_sdk.load(Ordering::SeqCst) < current_frame_generation_eruption_sdk {
                                let sdk_led_map = sdk_support::LED_MAP.read();
                                script::LED_MAP.write().par_chunks_exact_mut(constants::CANVAS_SIZE).for_each(|chunks| {
                                    alpha_blend(&sdk_led_map, chunks, 0.85);
                                });
                            }

                            #[cfg(not(target_os = "windows"))]
                            if ULEDS_SUPPORT_ACTIVE.load(Ordering::SeqCst) {
                                // blend the LED map of the Userspace LEDs support plugin
                                let uleds_led_map = uleds::LED_MAP.lock();

                                script::LED_MAP.write().par_chunks_exact_mut(constants::CANVAS_SIZE).for_each(|chunks| {
                                    alpha_blend(&uleds_led_map, chunks, 0.85);
                                });
                            }

                            // number of pending blend-ops should have reached zero by now
                            // this condition may occur during switching of profiles
                            let ops_pending = *COLOR_MAPS_READY_CONDITION.0.lock();
                            if ops_pending > 0 {
                                ratelimited::trace!(
                                    "Pending blend-ops before writing LED map to device: {}",
                                    ops_pending
                                );

                                drop_frame = true;
                            }

                            // apply global post-processing
                            let brightness = crate::BRIGHTNESS.load(Ordering::SeqCst);

                            let hsl = *crate::CANVAS_HSL.read();

                            let hue_value = hsl.0;
                            let saturation_value = hsl.1 / 100.0;
                            let lighten_value = hsl.2 / 100.0;
                            let brightness = brightness as f64 / 100.0 * 255.0;

                            script::LED_MAP.write().iter_mut().for_each(|color_val| {
                                let color = LinSrgba::new(
                                    color_val.r as f64 / 255.0,
                                    color_val.g as f64 / 255.0,
                                    color_val.b as f64 / 255.0,
                                    color_val.a as f64 / 255.0,
                                );

                                let color = Hsva::from_color(color);
                                let color = LinSrgba::from_color(
                                    color
                                        .shift_hue(hue_value)
                                        .saturate(saturation_value)
                                        .lighten(lighten_value)
                                    )
                                .into_components();

                                color_val.r = (color.0 * brightness).round() as u8;
                                color_val.g = (color.1 * brightness).round() as u8;
                                color_val.b = (color.2 * brightness).round() as u8;
                                color_val.a = 255_u8;
                            });

                            // send the final (combined) color map to all of the devices
                            if !drop_frame {
                                crate::DEVICES.read().par_iter().for_each(|(_handle, device)| {
                                    // NOTE: We may deadlock here, so be careful
                                    if let Some(mut dev) = device.try_write_for(constants::LOCK_CONTENDED_WAIT_MILLIS_SHORT) {
                                        if let Ok(is_initialized) = dev.is_initialized() {
                                            if is_initialized {
                                                if let Err(e) = dev.send_led_map(&script::LED_MAP.read()) {
                                                    ratelimited::error!("Error sending LED map to a device: {}", e);

                                                    ratelimited::warn!("Trying to unplug the failed device...");

                                                    dev.fail().unwrap_or_else(|e| {
                                                        error!("Could not mark a device as failed: {}", e);
                                                    });

                                                    // we need to terminate and then re-enter the main loop to update all global state
                                                    crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

                                                }
                                            } else {
                                                ratelimited::warn!("Skipping rendering to an uninitialized device");
                                            }
                                        } else {
                                            warn!("Could not query device status");
                                        }
                                    }
                                });

                                // update the current frame generation
                                saved_frame_generation.store(current_frame_generation, Ordering::SeqCst);

                                script::LAST_RENDERED_LED_MAP
                                    .write()
                                    .copy_from_slice(&script::LED_MAP.read());

                            fps_counter += 1;
                            }
                        }

                        // calculate and log fps each second
                        if fps_timer.elapsed().as_millis() >= 1000 {
                            debug!("FPS: {}", fps_counter);

                            fps_timer = Instant::now();
                            fps_counter = 0;
                        }
                    }
                },

                Err(e) => {
                    error!("Could not receive data: {}", e)
                }
            }
        }
    })?;

    Ok(())
}

#[inline]
fn alpha_blend(src: &[RGBA], dst: &mut [RGBA], factor: f32) {
    assert_eq!(src.len(), dst.len());

    dst.par_iter_mut()
        .zip(src.par_iter())
        .for_each(|(dst_pixel, src_pixel)| {
            let src_alpha = src_pixel.a as f32 / 255.0;
            let dst_alpha = dst_pixel.a as f32 / 255.0;

            let blend_alpha = (src_alpha * factor) + dst_alpha * (1.0 - factor);

            if blend_alpha > 0.0 {
                let blend_factor = (src_alpha * factor) / blend_alpha;

                let blended_pixel = RGBA {
                    r: ((src_pixel.r as f32 * blend_factor
                        + dst_pixel.r as f32 * (1.0 - blend_factor))
                        .round()) as u8,
                    g: ((src_pixel.g as f32 * blend_factor
                        + dst_pixel.g as f32 * (1.0 - blend_factor))
                        .round()) as u8,
                    b: ((src_pixel.b as f32 * blend_factor
                        + dst_pixel.b as f32 * (1.0 - blend_factor))
                        .round()) as u8,
                    a: (blend_alpha * 255.0).round() as u8,
                };

                *dst_pixel = blended_pixel;
            }
        });
}
