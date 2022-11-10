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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

#![allow(dead_code)]

use crate::{
    color_scheme::{ColorScheme, ColorSchemeExt},
    constants,
};
use dbus::blocking::Connection;
use dbus::nonblock;
use dbus_tokio::connection;
use std::{sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Returns a connection to the D-Bus system bus using the specified `path`
pub async fn dbus_system_bus(
    path: &str,
) -> Result<dbus::nonblock::Proxy<'_, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let proxy = nonblock::Proxy::new(
        "org.eruption",
        path,
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        conn,
    );

    Ok(proxy)
}

/// Returns a connection to the D-Bus session bus using the specified `path`
pub async fn dbus_session_bus<'a>(
    dest: &'a str,
    path: &'a str,
) -> Result<dbus::nonblock::Proxy<'a, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_session_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let proxy = nonblock::Proxy::new(
        dest,
        path,
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        conn,
    );

    Ok(proxy)
}

pub fn enable_ambient_effect() -> Result<()> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    proxy.enable_ambient_effect()?;

    Ok(())
}

pub fn disable_ambient_effect() -> Result<()> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    proxy.disable_ambient_effect()?;

    Ok(())
}

pub fn is_ambient_effect_enabled() -> Result<bool> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = proxy.ambient_effect()?;

    Ok(result)
}

pub fn set_parameter(
    profile_file: &str,
    script_file: &str,
    param_name: &str,
    value: &str,
) -> Result<()> {
    use profile::OrgEruptionProfile;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let _result = proxy.set_parameter(profile_file, script_file, param_name, value)?;

    Ok(())
}

// TODO: This currently fails with a dbus error, use util::get_slot_names() for now
/// Fetches all slot names
// pub fn get_slot_names() -> Result<Vec<String>> {
//     use slot::OrgEruptionSlot;

//     let conn = Connection::new_system()?;
//     let slot_proxy = conn.with_proxy(
//         "org.eruption",
//         "/org/eruption/slots",
//         Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
//     );

//     let result = slot_proxy.slot_names()?;

//     Ok(result)
// }

/// Get managed devices USB IDs from the eruption daemon
pub fn get_managed_devices() -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>)> {
    use status::OrgEruptionStatus;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/status",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = status_proxy.get_managed_devices()?;

    Ok(result)
}

pub fn get_color_schemes() -> Result<Vec<String>> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = config_proxy.get_color_schemes()?;

    Ok(result)
}

pub fn set_color_scheme(name: &str, color_scheme: &ColorScheme) -> Result<()> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let mut data = Vec::new();

    for index in 0..color_scheme.num_colors() {
        let result = color_scheme.color_rgba_at(index)?.to_linear_rgba_u8();

        data.push(result.0);
        data.push(result.1);
        data.push(result.2);
        data.push(result.3);
    }

    let _result = proxy.set_color_scheme(name, data)?;

    Ok(())
}

pub fn remove_color_scheme(name: &str) -> Result<bool> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = config_proxy.remove_color_scheme(name)?;

    Ok(result)
}

mod canvas {
    // This code was autogenerated with `dbus-codegen-rust -s -d org.eruption -p /org/eruption/canvas -m None`, see https://github.com/diwic/dbus-rs

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionCanvas {
        fn hue(&self) -> Result<f64, dbus::Error>;
        fn set_hue(&self, value: f64) -> Result<(), dbus::Error>;
        fn lightness(&self) -> Result<f64, dbus::Error>;
        fn set_lightness(&self, value: f64) -> Result<(), dbus::Error>;
        fn saturation(&self) -> Result<f64, dbus::Error>;
        fn set_saturation(&self, value: f64) -> Result<(), dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgEruptionCanvasHueChanged {
        pub hue: i64,
    }

    impl arg::AppendAll for OrgEruptionCanvasHueChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.hue, i);
        }
    }

    impl arg::ReadAll for OrgEruptionCanvasHueChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionCanvasHueChanged { hue: i.read()? })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionCanvasHueChanged {
        const NAME: &'static str = "HueChanged";
        const INTERFACE: &'static str = "org.eruption.Canvas";
    }

    #[derive(Debug)]
    pub struct OrgEruptionCanvasLightnessChanged {
        pub lightness: i64,
    }

    impl arg::AppendAll for OrgEruptionCanvasLightnessChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.lightness, i);
        }
    }

    impl arg::ReadAll for OrgEruptionCanvasLightnessChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionCanvasLightnessChanged {
                lightness: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionCanvasLightnessChanged {
        const NAME: &'static str = "LightnessChanged";
        const INTERFACE: &'static str = "org.eruption.Canvas";
    }

    #[derive(Debug)]
    pub struct OrgEruptionCanvasSaturationChanged {
        pub saturation: i64,
    }

    impl arg::AppendAll for OrgEruptionCanvasSaturationChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.saturation, i);
        }
    }

    impl arg::ReadAll for OrgEruptionCanvasSaturationChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionCanvasSaturationChanged {
                saturation: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionCanvasSaturationChanged {
        const NAME: &'static str = "SaturationChanged";
        const INTERFACE: &'static str = "org.eruption.Canvas";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionCanvas
        for blocking::Proxy<'a, C>
    {
        fn hue(&self) -> Result<f64, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Canvas",
                "Hue",
            )
        }

        fn lightness(&self) -> Result<f64, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Canvas",
                "Lightness",
            )
        }

        fn saturation(&self) -> Result<f64, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Canvas",
                "Saturation",
            )
        }

        fn set_hue(&self, value: f64) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Canvas",
                "Hue",
                value,
            )
        }

        fn set_lightness(&self, value: f64) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Canvas",
                "Lightness",
                value,
            )
        }

        fn set_saturation(&self, value: f64) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Canvas",
                "Saturation",
                value,
            )
        }
    }

    pub trait OrgFreedesktopDBusIntrospectable {
        fn introspect(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
    {
        fn introspect(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error>;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties: arg::PropMap,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusProperties for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(|r: (arg::PropMap,)| r.0)
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }
}

mod slot {
    // This code was autogenerated with `dbus-codegen-rust -s -d org.eruption -p /org/eruption/slot -m None`, see https://github.com/diwic/dbus-rs
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionSlot {
        fn get_slot_profiles(&self) -> Result<Vec<String>, dbus::Error>;
        fn switch_slot(&self, slot: u64) -> Result<bool, dbus::Error>;
        fn active_slot(&self) -> Result<u64, dbus::Error>;
        fn slot_names(&self) -> Result<Vec<String>, dbus::Error>;
        fn set_slot_names(&self, value: Vec<String>) -> Result<(), dbus::Error>;
    }

    impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgEruptionSlot
        for blocking::Proxy<'a, C>
    {
        fn get_slot_profiles(&self) -> Result<Vec<String>, dbus::Error> {
            self.method_call("org.eruption.Slot", "GetSlotProfiles", ())
                .map(|r: (Vec<String>,)| r.0)
        }

        fn switch_slot(&self, slot: u64) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Slot", "SwitchSlot", (slot,))
                .map(|r: (bool,)| r.0)
        }

        fn active_slot(&self) -> Result<u64, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Slot",
                "ActiveSlot",
            )
        }

        fn slot_names(&self) -> Result<Vec<String>, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Slot",
                "SlotNames",
            )
        }

        fn set_slot_names(&self, value: Vec<String>) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Slot",
                "SlotNames",
                value,
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgEruptionSlotActiveSlotChanged {
        pub new_slot: u64,
    }

    impl arg::AppendAll for OrgEruptionSlotActiveSlotChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.new_slot, i);
        }
    }

    impl arg::ReadAll for OrgEruptionSlotActiveSlotChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionSlotActiveSlotChanged {
                new_slot: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionSlotActiveSlotChanged {
        const NAME: &'static str = "ActiveSlotChanged";
        const INTERFACE: &'static str = "org.eruption.Slot";
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

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        >;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusProperties
        for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        > {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(
                |r: (
                    ::std::collections::HashMap<
                        String,
                        arg::Variant<Box<dyn arg::RefArg + 'static>>,
                    >,
                )| r.0,
            )
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties:
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }
}

mod profile {
    // This code was autogenerated with `dbus-codegen-rust -s -d org.eruption -p /org/eruption/profile -m None`, see https://github.com/diwic/dbus-rs

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionProfile {
        fn enum_profiles(&self) -> Result<Vec<(String, String)>, dbus::Error>;
        fn set_parameter(
            &self,
            profile_file: &str,
            script_file: &str,
            param_name: &str,
            value: &str,
        ) -> Result<bool, dbus::Error>;
        fn switch_profile(&self, filename: &str) -> Result<bool, dbus::Error>;
        fn active_profile(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionProfile
        for blocking::Proxy<'a, C>
    {
        fn enum_profiles(&self) -> Result<Vec<(String, String)>, dbus::Error> {
            self.method_call("org.eruption.Profile", "EnumProfiles", ())
                .map(|r: (Vec<(String, String)>,)| r.0)
        }

        fn set_parameter(
            &self,
            profile_file: &str,
            script_file: &str,
            param_name: &str,
            value: &str,
        ) -> Result<bool, dbus::Error> {
            self.method_call(
                "org.eruption.Profile",
                "SetParameter",
                (profile_file, script_file, param_name, value),
            )
            .map(|r: (bool,)| r.0)
        }

        fn switch_profile(&self, filename: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Profile", "SwitchProfile", (filename,))
                .map(|r: (bool,)| r.0)
        }

        fn active_profile(&self) -> Result<String, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Profile",
                "ActiveProfile",
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgEruptionProfileActiveProfileChanged {
        pub new_profile_name: String,
    }

    impl arg::AppendAll for OrgEruptionProfileActiveProfileChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.new_profile_name, i);
        }
    }

    impl arg::ReadAll for OrgEruptionProfileActiveProfileChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionProfileActiveProfileChanged {
                new_profile_name: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionProfileActiveProfileChanged {
        const NAME: &'static str = "ActiveProfileChanged";
        const INTERFACE: &'static str = "org.eruption.Profile";
    }

    #[derive(Debug)]
    pub struct OrgEruptionProfileProfilesChanged {}

    impl arg::AppendAll for OrgEruptionProfileProfilesChanged {
        fn append(&self, _: &mut arg::IterAppend) {}
    }

    impl arg::ReadAll for OrgEruptionProfileProfilesChanged {
        fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionProfileProfilesChanged {})
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionProfileProfilesChanged {
        const NAME: &'static str = "ProfilesChanged";
        const INTERFACE: &'static str = "org.eruption.Profile";
    }

    pub trait OrgFreedesktopDBusIntrospectable {
        fn introspect(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
    {
        fn introspect(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        >;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusProperties for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        > {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(
                |r: (
                    ::std::collections::HashMap<
                        String,
                        arg::Variant<Box<dyn arg::RefArg + 'static>>,
                    >,
                )| r.0,
            )
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties:
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }
}

mod config {
    // This code was autogenerated with `dbus-codegen-rust -s -d org.eruption -p /org/eruption/config`, see https://github.com/diwic/dbus-rs

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionConfig {
        fn get_color_schemes(&self) -> Result<Vec<String>, dbus::Error>;
        fn ping(&self) -> Result<bool, dbus::Error>;
        fn ping_privileged(&self) -> Result<bool, dbus::Error>;
        fn remove_color_scheme(&self, name: &str) -> Result<bool, dbus::Error>;
        fn set_color_scheme(&self, name: &str, data: Vec<u8>) -> Result<bool, dbus::Error>;
        fn write_file(&self, filename: &str, data: &str) -> Result<bool, dbus::Error>;
        fn brightness(&self) -> Result<i64, dbus::Error>;
        fn set_brightness(&self, value: i64) -> Result<(), dbus::Error>;
        fn enable_sfx(&self) -> Result<bool, dbus::Error>;
        fn set_enable_sfx(&self, value: bool) -> Result<(), dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgEruptionConfigBrightnessChanged {
        pub brightness: i64,
    }

    impl arg::AppendAll for OrgEruptionConfigBrightnessChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.brightness, i);
        }
    }

    impl arg::ReadAll for OrgEruptionConfigBrightnessChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionConfigBrightnessChanged {
                brightness: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionConfigBrightnessChanged {
        const NAME: &'static str = "BrightnessChanged";
        const INTERFACE: &'static str = "org.eruption.Config";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionConfig
        for blocking::Proxy<'a, C>
    {
        fn get_color_schemes(&self) -> Result<Vec<String>, dbus::Error> {
            self.method_call("org.eruption.Config", "GetColorSchemes", ())
                .map(|r: (Vec<String>,)| r.0)
        }

        fn ping(&self) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Config", "Ping", ())
                .map(|r: (bool,)| r.0)
        }

        fn ping_privileged(&self) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Config", "PingPrivileged", ())
                .map(|r: (bool,)| r.0)
        }

        fn remove_color_scheme(&self, name: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Config", "RemoveColorScheme", (name,))
                .map(|r: (bool,)| r.0)
        }

        fn set_color_scheme(&self, name: &str, data: Vec<u8>) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Config", "SetColorScheme", (name, data))
                .map(|r: (bool,)| r.0)
        }

        fn write_file(&self, filename: &str, data: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.eruption.Config", "WriteFile", (filename, data))
                .map(|r: (bool,)| r.0)
        }

        fn brightness(&self) -> Result<i64, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Config",
                "Brightness",
            )
        }

        fn enable_sfx(&self) -> Result<bool, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Config",
                "EnableSfx",
            )
        }

        fn set_brightness(&self, value: i64) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Config",
                "Brightness",
                value,
            )
        }

        fn set_enable_sfx(&self, value: bool) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.eruption.Config",
                "EnableSfx",
                value,
            )
        }
    }

    pub trait OrgFreedesktopDBusIntrospectable {
        fn introspect(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
    {
        fn introspect(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error>;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties: arg::PropMap,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusProperties for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(|r: (arg::PropMap,)| r.0)
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }
}

mod status {
    // This code was autogenerated with `dbus-codegen-rust -s -d org.eruption -p /org/eruption/status -m None`, see https://github.com/diwic/dbus-rs

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionStatus {
        fn get_led_colors(&self) -> Result<Vec<(u8, u8, u8, u8)>, dbus::Error>;
        fn get_managed_devices(
            &self,
        ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error>;
        fn running(&self) -> Result<bool, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionStatus
        for blocking::Proxy<'a, C>
    {
        fn get_led_colors(&self) -> Result<Vec<(u8, u8, u8, u8)>, dbus::Error> {
            self.method_call("org.eruption.Status", "GetLedColors", ())
                .map(|r: (Vec<(u8, u8, u8, u8)>,)| r.0)
        }

        fn get_managed_devices(
            &self,
        ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error> {
            self.method_call("org.eruption.Status", "GetManagedDevices", ())
                .map(|r: ((Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>),)| r.0)
        }

        fn running(&self) -> Result<bool, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.Status",
                "Running",
            )
        }
    }

    pub trait OrgFreedesktopDBusIntrospectable {
        fn introspect(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
    {
        fn introspect(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        >;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusProperties for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(
            &self,
            interface_name: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        > {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(
                |r: (
                    ::std::collections::HashMap<
                        String,
                        arg::Variant<Box<dyn arg::RefArg + 'static>>,
                    >,
                )| r.0,
            )
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties:
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }
}

mod fx_proxy {

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

    pub trait OrgEruptionFxProxyEffects {
        fn disable_ambient_effect(&self) -> Result<(), dbus::Error>;
        fn enable_ambient_effect(&self) -> Result<(), dbus::Error>;
        fn ambient_effect(&self) -> Result<bool, dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgEruptionFxProxyEffectsStatusChanged {
        pub event: String,
    }

    impl arg::AppendAll for OrgEruptionFxProxyEffectsStatusChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.event, i);
        }
    }

    impl arg::ReadAll for OrgEruptionFxProxyEffectsStatusChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgEruptionFxProxyEffectsStatusChanged { event: i.read()? })
        }
    }

    impl dbus::message::SignalArgs for OrgEruptionFxProxyEffectsStatusChanged {
        const NAME: &'static str = "StatusChanged";
        const INTERFACE: &'static str = "org.eruption.fx_proxy.Effects";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgEruptionFxProxyEffects for blocking::Proxy<'a, C>
    {
        fn disable_ambient_effect(&self) -> Result<(), dbus::Error> {
            self.method_call("org.eruption.fx_proxy.Effects", "DisableAmbientEffect", ())
        }

        fn enable_ambient_effect(&self) -> Result<(), dbus::Error> {
            self.method_call("org.eruption.fx_proxy.Effects", "EnableAmbientEffect", ())
        }

        fn ambient_effect(&self) -> Result<bool, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.eruption.fx_proxy.Effects",
                "AmbientEffect",
            )
        }
    }

    pub trait OrgFreedesktopDBusIntrospectable {
        fn introspect(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
    {
        fn introspect(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgFreedesktopDBusProperties {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error>;
        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error>;
    }

    #[derive(Debug)]
    pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
        pub interface_name: String,
        pub changed_properties: arg::PropMap,
        pub invalidated_properties: Vec<String>,
    }

    impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.interface_name, i);
            arg::RefArg::append(&self.changed_properties, i);
            arg::RefArg::append(&self.invalidated_properties, i);
        }
    }

    impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                interface_name: i.read()?,
                changed_properties: i.read()?,
                invalidated_properties: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
        const NAME: &'static str = "PropertiesChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
        OrgFreedesktopDBusProperties for blocking::Proxy<'a, C>
    {
        fn get(
            &self,
            interface_name: &str,
            property_name: &str,
        ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Get",
                (interface_name, property_name),
            )
            .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
        }

        fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "GetAll",
                (interface_name,),
            )
            .map(|r: (arg::PropMap,)| r.0)
        }

        fn set(
            &self,
            interface_name: &str,
            property_name: &str,
            value: arg::Variant<Box<dyn arg::RefArg>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.freedesktop.DBus.Properties",
                "Set",
                (interface_name, property_name, value),
            )
        }
    }
}
