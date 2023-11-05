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

use dbus::{arg::Iter, arg::IterAppend, ffidisp::Connection, message::SignalArgs};
use dbus_tree::{Access, EmitsChangedSignal, MethodErr, MethodResult, Signal};
use flume::Sender;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::*;

use crate::dbus_interface::Message;
use crate::{constants, plugins};

use super::{
    convenience::FactoryWithPermission, convenience::InterfaceAddend,
    convenience::PropertyWithPermission, perms::Permission, Factory, Interface, MethodInfo,
    Property,
};

#[derive(Clone)]
pub struct SlotInterface {
    dbus_tx: Arc<Sender<Message>>,
    conn: Arc<Connection>,
    pub active_slot_changed_signal: Arc<Signal<()>>,
    active_slot_property: Arc<Property>,
}

impl SlotInterface {
    pub fn new(f: &Factory, dbus_tx: Arc<Sender<Message>>, conn: Arc<Connection>) -> Self {
        let active_slot_changed_signal =
            Arc::new(f.signal("ActiveSlotChanged", ()).sarg::<u64, _>("slot"));

        let active_slot_property = Arc::new(
            f.property::<u64, _>("ActiveSlot", ())
                .emits_changed(EmitsChangedSignal::Const)
                .on_get_with_permission(Permission::Monitor, get_active_slot),
        );

        Self {
            dbus_tx,
            conn,
            active_slot_changed_signal,
            active_slot_property,
        }
    }
}

impl InterfaceAddend for SlotInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        let conn = self.conn.clone();
        let dbus_tx = self.dbus_tx.clone();
        let active_slot_property = self.active_slot_property.clone();

        interface
            .add_s(self.active_slot_changed_signal.clone())
            .add_p(self.active_slot_property.clone())
            .add_m(
                f.method_with_permission("SwitchSlot", Permission::Settings, move |m| {
                    switch_slot(m, &conn, &dbus_tx, &active_slot_property)
                })
                .inarg::<u64, _>("slot")
                .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission("GetSlotProfiles", Permission::Monitor, get_slot_profiles)
                    .outarg::<Vec<String>, _>("values"),
            )
            .add_p(
                f.property::<Vec<String>, _>("SlotNames", ())
                    .access(Access::ReadWrite)
                    .emits_changed(EmitsChangedSignal::True)
                    .auto_emit_on_set(true)
                    .on_get_with_permission(Permission::Monitor, get_slot_names)
                    .on_set_with_permission(Permission::Settings, set_slot_names),
            )
    }
}

fn get_active_slot(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    let result = crate::ACTIVE_SLOT.load(Ordering::SeqCst) as u64;
    i.append(result);

    Ok(())
}

fn switch_slot(
    m: &MethodInfo,
    conn: &Arc<Connection>,
    dbus_tx: &Arc<Sender<Message>>,
    active_slot_property: &Arc<Property>,
) -> MethodResult {
    let n: u64 = m.msg.read1()?;

    if n as usize >= constants::NUM_SLOTS {
        Err(MethodErr::failed("Slot index out of bounds"))
    } else {
        dbus_tx
            .send(Message::SwitchSlot(n as usize))
            .unwrap_or_else(|e| error!("Could not send a pending D-Bus event: {}", e));

        // reset the audio backend, it will be enabled again if needed
        #[cfg(not(target_os = "windows"))]
        plugins::audio::reset_audio_backend();

        let mut changed_properties = Vec::new();
        active_slot_property.add_propertieschanged(
            &mut changed_properties,
            &super::INTERFACE_ROOT,
            || Box::new(n),
        );
        if !changed_properties.is_empty() {
            let msg = changed_properties
                .first()
                .unwrap()
                .to_emit_message(&super::SLOT_PATH);
            conn.send(msg).unwrap();
        }
        let s = true;
        Ok(vec![m.msg.method_return().append1(s)])
    }
}

fn get_slot_profiles(m: &MethodInfo) -> MethodResult {
    let s: Vec<String> = crate::SLOT_PROFILES
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    Ok(vec![m.msg.method_return().append1(s)])
}

fn get_slot_names(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    let s = crate::SLOT_NAMES.read().unwrap();
    i.append(&*s);

    Ok(())
}

fn set_slot_names(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    let n: Vec<String> = i.read()?;

    if n.len() >= constants::NUM_SLOTS {
        *crate::SLOT_NAMES.write().unwrap() = n;

        Ok(())
    } else {
        Err(MethodErr::failed("Invalid number of elements"))
    }
}
