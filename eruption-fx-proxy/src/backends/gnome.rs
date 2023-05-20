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

use dbus::blocking::Connection;
use image::ImageBuffer;

use std::time::Duration;

use crate::constants;

use super::{Backend, BackendData};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Clone)]
pub struct GnomeBackend {
    pub failed: bool,
}

impl GnomeBackend {
    pub fn new() -> Result<Self> {
        Ok(Self { failed: true })
    }
}

impl Backend for GnomeBackend {
    fn initialize(&mut self) -> Result<()> {
        self.failed = true;

        let _opts = crate::OPTIONS.read().as_ref().unwrap().clone();

        // if we made it up to here, the initialization succeeded
        self.failed = false;

        Ok(())
    }

    fn get_id(&self) -> String {
        "gnome".to_string()
    }

    fn get_name(&self) -> String {
        "GNOME".to_string()
    }

    fn get_description(&self) -> String {
        "Capture the screen's content from a GNOME session".to_string()
    }

    fn is_failed(&self) -> bool {
        self.failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    fn poll(&mut self) -> Result<BackendData> {
        // use screenshot::OrgGnomeShellScreenshot;

        let conn = Connection::new_session()?;
        let _screenshot_proxy = conn.with_proxy(
            "org.gnome.Shell.Screenshot",
            "/org/gnome/Shell/Screenshot",
            Duration::from_millis(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        /*
        let result =
            screenshot_proxy.screenshot(true, true, "/tmp/eruption-netfx/screenshot.png")?;

        icecream::ic!(result);

        let result =
            super::utils::process_image_file("/tmp/eruption-netfx/screenshot.png", &device)?;
        */

        let result = ImageBuffer::new(0, 0);

        Ok(result)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

mod screenshot {
    // This code was autogenerated with `dbus-codegen-rust -d org.gnome.Shell.Screenshot -p /org/gnome/Shell/Screenshot`, see https://github.com/diwic/dbus-rs
    use dbus;
    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

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

    pub trait OrgFreedesktopDBusPeer {
        fn ping(&self) -> Result<(), dbus::Error>;
        fn get_machine_id(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgFreedesktopDBusPeer
        for blocking::Proxy<'a, C>
    {
        fn ping(&self) -> Result<(), dbus::Error> {
            self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
        }

        fn get_machine_id(&self) -> Result<String, dbus::Error> {
            self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
                .map(|r: (String,)| r.0)
        }
    }

    pub trait OrgGnomeShellScreenshot {
        fn screenshot(
            &self,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error>;
        fn screenshot_window(
            &self,
            include_frame: bool,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error>;
        fn screenshot_area(
            &self,
            x_: i32,
            y_: i32,
            width: i32,
            height: i32,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error>;
        fn pick_color(&self) -> Result<arg::PropMap, dbus::Error>;
        fn flash_area(&self, x_: i32, y_: i32, width: i32, height: i32) -> Result<(), dbus::Error>;
        fn select_area(&self) -> Result<(i32, i32, i32, i32), dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgGnomeShellScreenshot
        for blocking::Proxy<'a, C>
    {
        fn screenshot(
            &self,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Screenshot",
                "Screenshot",
                (include_cursor, flash, filename),
            )
        }

        fn screenshot_window(
            &self,
            include_frame: bool,
            include_cursor: bool,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Screenshot",
                "ScreenshotWindow",
                (include_frame, include_cursor, flash, filename),
            )
        }

        fn screenshot_area(
            &self,
            x_: i32,
            y_: i32,
            width: i32,
            height: i32,
            flash: bool,
            filename: &str,
        ) -> Result<(bool, String), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Screenshot",
                "ScreenshotArea",
                (x_, y_, width, height, flash, filename),
            )
        }

        fn pick_color(&self) -> Result<arg::PropMap, dbus::Error> {
            self.method_call("org.gnome.Shell.Screenshot", "PickColor", ())
                .map(|r: (arg::PropMap,)| r.0)
        }

        fn flash_area(&self, x_: i32, y_: i32, width: i32, height: i32) -> Result<(), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Screenshot",
                "FlashArea",
                (x_, y_, width, height),
            )
        }

        fn select_area(&self) -> Result<(i32, i32, i32, i32), dbus::Error> {
            self.method_call("org.gnome.Shell.Screenshot", "SelectArea", ())
        }
    }
}
