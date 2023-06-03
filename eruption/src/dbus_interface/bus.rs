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

// This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.DBus -p /org/freedesktop/DBus/Bus -m None`, see https://github.com/diwic/dbus-rs

use dbus::arg;
use dbus::blocking;

pub trait OrgFreedesktopDBus {
    fn hello(&self) -> Result<String, dbus::Error>;
    fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
    fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
    fn update_activation_environment(
        &self,
        arg0: ::std::collections::HashMap<&str, &str>,
    ) -> Result<(), dbus::Error>;
    fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error>;
    fn list_names(&self) -> Result<Vec<String>, dbus::Error>;
    fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error>;
    fn add_match(&self, arg0: &str) -> Result<(), dbus::Error>;
    fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error>;
    fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error>;
    fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error>;
    fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error>;
    fn get_connection_selinux_security_context(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error>;
    fn reload_config(&self) -> Result<(), dbus::Error>;
    fn get_id(&self) -> Result<String, dbus::Error>;
    fn get_connection_credentials(
        &self,
        arg0: &str,
    ) -> Result<
        ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
        dbus::Error,
    >;
    fn features(&self) -> Result<Vec<String>, dbus::Error>;
    fn interfaces(&self) -> Result<Vec<String>, dbus::Error>;
}

impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBus
    for blocking::Proxy<'a, C>
{
    fn hello(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "Hello", ())
            .map(|r: (String,)| r.0)
    }

    fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "RequestName", (arg0, arg1))
            .map(|r: (u32,)| r.0)
    }

    fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ReleaseName", (arg0,))
            .map(|r: (u32,)| r.0)
    }

    fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "StartServiceByName", (arg0, arg1))
            .map(|r: (u32,)| r.0)
    }

    fn update_activation_environment(
        &self,
        arg0: ::std::collections::HashMap<&str, &str>,
    ) -> Result<(), dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "UpdateActivationEnvironment",
            (arg0,),
        )
    }

    fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "NameHasOwner", (arg0,))
            .map(|r: (bool,)| r.0)
    }

    fn list_names(&self) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListNames", ())
            .map(|r: (Vec<String>,)| r.0)
    }

    fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListActivatableNames", ())
            .map(|r: (Vec<String>,)| r.0)
    }

    fn add_match(&self, arg0: &str) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "AddMatch", (arg0,))
    }

    fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "RemoveMatch", (arg0,))
    }

    fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetNameOwner", (arg0,))
            .map(|r: (String,)| r.0)
    }

    fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListQueuedOwners", (arg0,))
            .map(|r: (Vec<String>,)| r.0)
    }

    fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetConnectionUnixUser", (arg0,))
            .map(|r: (u32,)| r.0)
    }

    fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "GetConnectionUnixProcessID",
            (arg0,),
        )
        .map(|r: (u32,)| r.0)
    }

    fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetAdtAuditSessionData", (arg0,))
            .map(|r: (Vec<u8>,)| r.0)
    }

    fn get_connection_selinux_security_context(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "GetConnectionSELinuxSecurityContext",
            (arg0,),
        )
        .map(|r: (Vec<u8>,)| r.0)
    }

    fn reload_config(&self) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ReloadConfig", ())
    }

    fn get_id(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetId", ())
            .map(|r: (String,)| r.0)
    }

    fn get_connection_credentials(
        &self,
        arg0: &str,
    ) -> Result<
        ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
        dbus::Error,
    > {
        self.method_call("org.freedesktop.DBus", "GetConnectionCredentials", (arg0,))
            .map(
                |r: (
                    ::std::collections::HashMap<
                        String,
                        arg::Variant<Box<dyn arg::RefArg + 'static>>,
                    >,
                )| r.0,
            )
    }

    fn features(&self) -> Result<Vec<String>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.DBus",
            "Features",
        )
    }

    fn interfaces(&self) -> Result<Vec<String>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.DBus",
            "Interfaces",
        )
    }
}

#[derive(Debug)]
pub struct OrgFreedesktopDBusNameOwnerChanged {
    pub arg0: String,
    pub arg1: String,
    pub arg2: String,
}

impl arg::AppendAll for OrgFreedesktopDBusNameOwnerChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
        arg::RefArg::append(&self.arg1, i);
        arg::RefArg::append(&self.arg2, i);
    }
}

impl arg::ReadAll for OrgFreedesktopDBusNameOwnerChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopDBusNameOwnerChanged {
            arg0: i.read()?,
            arg1: i.read()?,
            arg2: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopDBusNameOwnerChanged {
    const NAME: &'static str = "NameOwnerChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

#[derive(Debug)]
pub struct OrgFreedesktopDBusNameLost {
    pub arg0: String,
}

impl arg::AppendAll for OrgFreedesktopDBusNameLost {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
    }
}

impl arg::ReadAll for OrgFreedesktopDBusNameLost {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopDBusNameLost { arg0: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopDBusNameLost {
    const NAME: &'static str = "NameLost";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

#[derive(Debug)]
pub struct OrgFreedesktopDBusNameAcquired {
    pub arg0: String,
}

impl arg::AppendAll for OrgFreedesktopDBusNameAcquired {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
    }
}

impl arg::ReadAll for OrgFreedesktopDBusNameAcquired {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopDBusNameAcquired { arg0: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopDBusNameAcquired {
    const NAME: &'static str = "NameAcquired";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

pub trait OrgFreedesktopDBusIntrospectable {
    fn introspect(&self) -> Result<String, dbus::Error>;
}

impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusIntrospectable
    for blocking::Proxy<'a, C>
{
    fn introspect(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
            .map(|r: (String,)| r.0)
    }
}

pub trait OrgFreedesktopDBusPeer {
    fn get_machine_id(&self) -> Result<String, dbus::Error>;
    fn ping(&self) -> Result<(), dbus::Error>;
}

impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusPeer
    for blocking::Proxy<'a, C>
{
    fn get_machine_id(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
            .map(|r: (String,)| r.0)
    }

    fn ping(&self) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
    }
}