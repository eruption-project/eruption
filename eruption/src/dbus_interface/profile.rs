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

use dbus::{arg::IterAppend, ffidisp::Connection, message::SignalArgs};
use dbus_tree::{EmitsChangedSignal, MethodErr, MethodResult, Signal};
use flume::Sender;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::*;

use crate::dbus_interface::Message;
use crate::{plugins, profiles, scripting::parameters, scripting::parameters_util};

use super::{
    convenience::FactoryWithPermission, convenience::InterfaceAddend,
    convenience::PropertyWithPermission, perms::Permission, Factory, Interface, MethodInfo,
    Property,
};

#[derive(Clone)]
pub struct ProfileInterface {
    dbus_tx: Arc<Sender<Message>>,
    conn: Arc<Connection>,
    pub profiles_changed_signal: Arc<Signal<()>>,
    pub active_profile_changed_signal: Arc<Signal<()>>,
    active_profile_property: Arc<Property>,
}

impl ProfileInterface {
    pub fn new(f: &Factory, dbus_tx: Arc<Sender<Message>>, conn: Arc<Connection>) -> Self {
        let profiles_changed_signal = Arc::new(f.signal("ProfilesChanged", ()));

        let active_profile_changed_signal = Arc::new(
            f.signal("ActiveProfileChanged", ())
                .sarg::<String, _>("profile_name"),
        );

        let active_profile_property = Arc::new(
            f.property::<String, _>("ActiveProfile", ())
                .emits_changed(EmitsChangedSignal::Const)
                .on_get_with_permission(Permission::Monitor, get_active_profile),
        );

        Self {
            dbus_tx,
            conn,
            profiles_changed_signal,
            active_profile_changed_signal,
            active_profile_property,
        }
    }
}

impl InterfaceAddend for ProfileInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        let conn = self.conn.clone();
        let dbus_tx = self.dbus_tx.clone();
        let active_profile_property = self.active_profile_property.clone();

        interface
            .add_s(self.profiles_changed_signal.clone())
            .add_s(self.active_profile_changed_signal.clone())
            .add_p(self.active_profile_property.clone())
            .add_m(
                f.method_with_permission("SwitchProfile", Permission::Settings, move |m| {
                    switch_profile(m, &conn, &dbus_tx, &active_profile_property)
                })
                .inarg::<&str, _>("filename")
                .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission("EnumProfiles", Permission::Monitor, enum_profiles)
                    .outarg::<Vec<(String, String)>, _>("profiles"),
            )
            .add_m(
                f.method_with_permission("SetParameter", Permission::Settings, set_parameter)
                    .inarg::<&str, _>("profile_file")
                    .inarg::<&str, _>("script_file")
                    .inarg::<&str, _>("param_name")
                    .inarg::<&str, _>("value")
                    .outarg::<bool, _>("status"),
            )
    }
}

fn get_active_profile(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    let result = crate::ACTIVE_PROFILE.read().unwrap();

    result
        .as_ref()
        .map(|p| {
            i.append(&*p.profile_file.to_string_lossy());
        })
        .ok_or_else(|| MethodErr::failed("Method failed"))
}

fn switch_profile(
    m: &MethodInfo,
    conn: &Arc<Connection>,
    dbus_tx: &Arc<Sender<Message>>,
    active_profile_property: &Arc<Property>,
) -> MethodResult {
    let n: &str = m.msg.read1()?;

    dbus_tx
        .send(Message::SwitchProfile(PathBuf::from(n)))
        .unwrap_or_else(|e| error!("Could not send a pending D-Bus event: {}", e));

    // reset the audio backend, it will be enabled again if needed
    #[cfg(not(target_os = "windows"))]
    plugins::audio::reset_audio_backend();

    let mut changed_properties = Vec::new();
    active_profile_property.add_propertieschanged(
        &mut changed_properties,
        &super::INTERFACE_ROOT,
        || Box::new(n.to_owned()),
    );

    if !changed_properties.is_empty() {
        let msg = changed_properties
            .first()
            .unwrap()
            .to_emit_message(&super::PROFILE_PATH);
        conn.send(msg).unwrap();
    }

    Ok(vec![m.msg.method_return().append1(true)])
}

fn enum_profiles(m: &MethodInfo) -> MethodResult {
    let mut s: Vec<(String, String)> = profiles::get_profiles()
        .unwrap_or_else(|_| vec![])
        .iter()
        .map(|profile| {
            (
                profile.name.clone(),
                profile.profile_file.to_string_lossy().to_string(),
            )
        })
        .collect();

    s.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

    Ok(vec![m.msg.method_return().append1(s)])
}

fn set_parameter(m: &MethodInfo) -> MethodResult {
    let (profile_file, script_file, param_name, value): (&str, &str, &str, &str) = m.msg.read4()?;

    debug!(
        "Setting parameter {}:{} {} to '{}'",
        &profile_file, &script_file, &param_name, &value
    );

    let applied = parameters_util::apply_parameters(
        profile_file,
        script_file,
        &[parameters::UntypedParameter {
            name: param_name.to_string(),
            value: value.to_string(),
        }],
    );
    match applied {
        Ok(()) => Ok(vec![m.msg.method_return().append1(true)]),
        Err(err) => {
            debug!("Could not set parameter: {}", err);
            Err(MethodErr::invalid_arg(&value))
        }
    }
}
