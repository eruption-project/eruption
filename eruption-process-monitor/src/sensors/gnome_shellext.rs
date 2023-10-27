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

use async_trait::async_trait;
use core::sync::atomic::Ordering;
use flume::Sender;
use lazy_static::lazy_static;
use nix::{
    errno::Errno,
    fcntl::{self, OFlag},
    poll::{self, PollFd, PollFlags},
    sys::stat::Mode,
    unistd::{self},
};
use parking_lot::Mutex;
use serde::Deserialize;
use std::os::fd::AsRawFd;
use std::{env, path::PathBuf};
use std::{
    io,
    os::fd::{FromRawFd, OwnedFd},
};
use std::{sync::Arc, thread};

use crate::constants;
use crate::QUIT;

use super::{Sensor, SensorConfiguration, SENSORS_CONFIGURATION};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Metadata of the GNOME 4x shell extension
// const GNOME4X_TOPLEVEL_WINDOW_PROPS_EXTENSION_META: &str = r#"
//     {
//         "name": "Eruption Sensor (eruption-process-monitor)",
//         "description": "Sensor Extension for the Eruption Realtime RGB LED Driver for Linux",
//         "uuid": "eruption-sensor@x3n0m0rph59.org",
//         "version": "1",
//         "shell-version": [ "45" ]
//     }
// "#;

/// JavaScript code that fetches the properties of the top-level window from GNOME 4x
// const GNOME4X_TOPLEVEL_WINDOW_PROPS_EXTENSION: &str = r#"
// "#;

#[derive(Debug, thiserror::Error)]
pub enum GnomeShellExtensionSensorError {
    // #[error("Sensor error: {description}")]
    // SensorError { description: String },
    #[error("Operation not supported")]
    NotSupported,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GnomeShellExtSensorData {
    pub window_title: String,
    // pub window_instance: String,
    pub window_class: String,
    // pub pid: i32,
}

impl super::SensorData for GnomeShellExtSensorData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl super::WindowSensorData for GnomeShellExtSensorData {
    fn window_name(&self) -> Option<&str> {
        Some(&self.window_title)
    }

    fn window_instance(&self) -> Option<&str> {
        // Some(&self.window_instance)

        None
    }

    fn window_class(&self) -> Option<&str> {
        Some(&self.window_class)
    }
}

lazy_static! {
    /// Events tx to the main thread
    static ref THREAD_TX: Arc<Mutex<Option<Sender<GnomeShellExtSensorData>>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone)]
pub struct GnomeShellExtensionSensor {
    pub is_failed: bool,
}

impl GnomeShellExtensionSensor {
    pub fn new() -> Self {
        Self { is_failed: false }
    }

    // pub fn setup_pipe(&self) -> Result<()> {
    //     let fifo_path = PathBuf::from(
    //         env::var("XDG_RUNTIME_DIR")
    //             .map(|v| v.to_owned().to_string())
    //             .unwrap_or_else(|_e| "/run/user/1000/".to_string()),
    //     )
    //     .join("eruption-sensor");

    //     if !util::file_exists(&fifo_path) {
    //         mkfifo(&fifo_path, Mode::S_IRUSR | Mode::S_IWUSR)?;
    //     }

    //     Ok(())
    // }

    pub fn spawn_events_thread(&mut self, tx: Sender<GnomeShellExtSensorData>) -> Result<()> {
        *THREAD_TX.lock() = Some(tx);

        let fifo_path = PathBuf::from(
            env::var("XDG_RUNTIME_DIR")
                .map(|v| v.to_owned().to_string())
                .unwrap_or_else(|_e| "/run/user/1000/".to_string()),
        )
        .join("eruption-sensor");

        thread::Builder::new()
            .name("shell-events".to_owned())
            .spawn(move || -> Result<()> {
                // Open the FIFO for reading (you can use other flags based on your requirements)
                let fifo_fd = unsafe {
                    OwnedFd::from_raw_fd(fcntl::open(&fifo_path, OFlag::O_RDONLY, Mode::empty())?)
                };

                // Set up the pollfd structure for monitoring the FIFO
                let poll_fd = PollFd::new(&fifo_fd, PollFlags::POLLIN);

                'EVENTS_LOOP: loop {
                    // check if we shall terminate the thread
                    if QUIT.load(Ordering::SeqCst) {
                        break 'EVENTS_LOOP Ok(());
                    }

                    // Create a vector of pollfd structures
                    let mut poll_fds = vec![poll_fd];

                    // Perform the poll operation
                    match poll::poll(&mut poll_fds, constants::POLL_TIMEOUT as i32) {
                        Ok(num_fds) => {
                            if num_fds == 0 {
                                tracing::trace!("No events received within the specified timeout");
                            } else if poll_fd.revents().is_some() {
                                // The named pipe is ready for reading
                                let mut buffer = [0; 4096 * 4];

                                match unistd::read(fifo_fd.as_raw_fd(), &mut buffer) {
                                    Ok(bytes_read) => {
                                        if bytes_read > 0 {
                                            let data = &buffer[..bytes_read];
                                            let data_str = String::from_utf8_lossy(data);
                                            tracing::debug!(
                                                "Received data from the FIFO: {}",
                                                data_str
                                            );

                                            if !data_str.trim().is_empty() {
                                                match serde_json::from_str::<
                                                    GnomeShellExtSensorData,
                                                >(
                                                    &data_str
                                                ) {
                                                    Ok(result) => {
                                                        tracing::debug!(
                                                            "Sensor data: {result:#?}"
                                                        );

                                            THREAD_TX
                                                        .lock()
                                                        .as_ref()
                                                        .unwrap()
                                                        .send(result)
                                                        .unwrap_or_else(|e| {
                                                            tracing::error!("Could not send on a channel: {}", e)
                                                        });
                                                    }

                                                    Err(e) => {
                                                        tracing::error!(
                                                            "Error parsing sensor data: {e}"
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    Err(_e) => {
                                        tracing::error!("Error reading from the FIFO");
                                    }
                                }
                            }
                        }

                        Err(e) => {
                            if e == Errno::EINTR {
                                // The poll system call was interrupted, possibly by a signal
                                tracing::debug!("Interrupted system call");

                                continue;
                            } else {
                                break 'EVENTS_LOOP Err(io::Error::last_os_error().into());
                            }
                        }
                    }
                }
            })?;

        Ok(())
    }
}

#[async_trait]
impl Sensor for GnomeShellExtensionSensor {
    fn get_id(&self) -> String {
        "gnome-shellext".to_string()
    }

    fn get_name(&self) -> String {
        "Eruption Sensor GNOME 4x Shell Extension".to_string()
    }

    fn get_description(&self) -> String {
        "Watches the state of windows on a GNOME desktop using the Eruption Sensor GNOME shell extension"
            .to_string()
    }

    fn get_usage_example(&self) -> String {
        r#"
gnome-shellext:
rules add window-[class|instance|name] <regex> [<profile-name.profile>|<slot number>]

rules add window-name '.*YouTube.*Mozilla Firefox' /var/lib/eruption/profiles/profile1.profile
rules add window-instance gnome-calculator 2
"#
        .to_string()
    }

    fn initialize(&mut self) -> Result<()> {
        // self.setup_pipe()?;

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        SENSORS_CONFIGURATION
            .read()
            .contains(&SensorConfiguration::EnableGnomeShellExt)
    }

    fn is_pollable(&self) -> bool {
        false
    }

    fn is_failed(&self) -> bool {
        self.is_failed
    }

    fn set_failed(&mut self, _failed: bool) {
        // no op
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        Err(GnomeShellExtensionSensorError::NotSupported.into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/* mod gnome {
    // This code was autogenerated with `dbus-codegen-rust -d org.gnome.Shell -p /org/gnome/Shell -m None`, see https://github.com/diwic/dbus-rs

    #[allow(unused_imports)]
    use dbus::arg;
    use dbus::blocking;

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

    pub trait OrgGnomeShell {
        fn eval(&self, script: &str) -> Result<(bool, String), dbus::Error>;
        fn focus_search(&self) -> Result<(), dbus::Error>;
        fn show_osd(
            &self,
            params: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error>;
        fn show_monitor_labels(
            &self,
            params: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error>;
        fn hide_monitor_labels(&self) -> Result<(), dbus::Error>;
        fn focus_app(&self, id: &str) -> Result<(), dbus::Error>;
        fn show_applications(&self) -> Result<(), dbus::Error>;
        fn grab_accelerator(
            &self,
            accelerator: &str,
            mode_flags: u32,
            grab_flags: u32,
        ) -> Result<u32, dbus::Error>;
        fn grab_accelerators(
            &self,
            accelerators: Vec<(&str, u32, u32)>,
        ) -> Result<Vec<u32>, dbus::Error>;
        fn ungrab_accelerator(&self, action: u32) -> Result<bool, dbus::Error>;
        fn ungrab_accelerators(&self, action: Vec<u32>) -> Result<bool, dbus::Error>;
        fn mode(&self) -> Result<String, dbus::Error>;
        fn overview_active(&self) -> Result<bool, dbus::Error>;
        fn set_overview_active(&self, value: bool) -> Result<(), dbus::Error>;
        fn shell_version(&self) -> Result<String, dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgGnomeShell
        for blocking::Proxy<'a, C>
    {
        fn eval(&self, script: &str) -> Result<(bool, String), dbus::Error> {
            self.method_call("org.gnome.Shell", "Eval", (script,))
        }

        fn focus_search(&self) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "FocusSearch", ())
        }

        fn show_osd(
            &self,
            params: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "ShowOSD", (params,))
        }

        fn show_monitor_labels(
            &self,
            params: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "ShowMonitorLabels", (params,))
        }

        fn hide_monitor_labels(&self) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "HideMonitorLabels", ())
        }

        fn focus_app(&self, id: &str) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "FocusApp", (id,))
        }

        fn show_applications(&self) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell", "ShowApplications", ())
        }

        fn grab_accelerator(
            &self,
            accelerator: &str,
            mode_flags: u32,
            grab_flags: u32,
        ) -> Result<u32, dbus::Error> {
            self.method_call(
                "org.gnome.Shell",
                "GrabAccelerator",
                (accelerator, mode_flags, grab_flags),
            )
            .map(|r: (u32,)| r.0)
        }

        fn grab_accelerators(
            &self,
            accelerators: Vec<(&str, u32, u32)>,
        ) -> Result<Vec<u32>, dbus::Error> {
            self.method_call("org.gnome.Shell", "GrabAccelerators", (accelerators,))
                .map(|r: (Vec<u32>,)| r.0)
        }

        fn ungrab_accelerator(&self, action: u32) -> Result<bool, dbus::Error> {
            self.method_call("org.gnome.Shell", "UngrabAccelerator", (action,))
                .map(|r: (bool,)| r.0)
        }

        fn ungrab_accelerators(&self, action: Vec<u32>) -> Result<bool, dbus::Error> {
            self.method_call("org.gnome.Shell", "UngrabAccelerators", (action,))
                .map(|r: (bool,)| r.0)
        }

        fn mode(&self) -> Result<String, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.gnome.Shell",
                "Mode",
            )
        }

        fn overview_active(&self) -> Result<bool, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.gnome.Shell",
                "OverviewActive",
            )
        }

        fn shell_version(&self) -> Result<String, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.gnome.Shell",
                "ShellVersion",
            )
        }

        fn set_overview_active(&self, value: bool) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.gnome.Shell",
                "OverviewActive",
                value,
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgGnomeShellAcceleratorActivated {
        pub action: u32,
        pub parameters:
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
    }

    impl arg::AppendAll for OrgGnomeShellAcceleratorActivated {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.action, i);
            arg::RefArg::append(&self.parameters, i);
        }
    }

    impl arg::ReadAll for OrgGnomeShellAcceleratorActivated {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgGnomeShellAcceleratorActivated {
                action: i.read()?,
                parameters: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgGnomeShellAcceleratorActivated {
        const NAME: &'static str = "AcceleratorActivated";
        const INTERFACE: &'static str = "org.gnome.Shell";
    }

    pub trait OrgGnomeShellExtensions {
        fn list_extensions(
            &self,
        ) -> Result<
            ::std::collections::HashMap<
                String,
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            >,
            dbus::Error,
        >;
        fn get_extension_info(
            &self,
            uuid: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        >;
        fn get_extension_errors(&self, uuid: &str) -> Result<Vec<String>, dbus::Error>;
        fn install_remote_extension(&self, uuid: &str) -> Result<String, dbus::Error>;
        fn uninstall_extension(&self, uuid: &str) -> Result<bool, dbus::Error>;
        fn reload_extension(&self, uuid: &str) -> Result<(), dbus::Error>;
        fn enable_extension(&self, uuid: &str) -> Result<bool, dbus::Error>;
        fn disable_extension(&self, uuid: &str) -> Result<bool, dbus::Error>;
        fn launch_extension_prefs(&self, uuid: &str) -> Result<(), dbus::Error>;
        fn open_extension_prefs(
            &self,
            uuid: &str,
            parent_window: &str,
            options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error>;
        fn check_for_updates(&self) -> Result<(), dbus::Error>;
        fn shell_version(&self) -> Result<String, dbus::Error>;
        fn user_extensions_enabled(&self) -> Result<bool, dbus::Error>;
        fn set_user_extensions_enabled(&self, value: bool) -> Result<(), dbus::Error>;
    }

    impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgGnomeShellExtensions
        for blocking::Proxy<'a, C>
    {
        fn list_extensions(
            &self,
        ) -> Result<
            ::std::collections::HashMap<
                String,
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            >,
            dbus::Error,
        > {
            self.method_call("org.gnome.Shell.Extensions", "ListExtensions", ())
                .map(
                    |r: (
                        ::std::collections::HashMap<
                            String,
                            ::std::collections::HashMap<
                                String,
                                arg::Variant<Box<dyn arg::RefArg + 'static>>,
                            >,
                        >,
                    )| r.0,
                )
        }

        fn get_extension_info(
            &self,
            uuid: &str,
        ) -> Result<
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            dbus::Error,
        > {
            self.method_call("org.gnome.Shell.Extensions", "GetExtensionInfo", (uuid,))
                .map(
                    |r: (
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    )| r.0,
                )
        }

        fn get_extension_errors(&self, uuid: &str) -> Result<Vec<String>, dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "GetExtensionErrors", (uuid,))
                .map(|r: (Vec<String>,)| r.0)
        }

        fn install_remote_extension(&self, uuid: &str) -> Result<String, dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Extensions",
                "InstallRemoteExtension",
                (uuid,),
            )
            .map(|r: (String,)| r.0)
        }

        fn uninstall_extension(&self, uuid: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "UninstallExtension", (uuid,))
                .map(|r: (bool,)| r.0)
        }

        fn reload_extension(&self, uuid: &str) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "ReloadExtension", (uuid,))
        }

        fn enable_extension(&self, uuid: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "EnableExtension", (uuid,))
                .map(|r: (bool,)| r.0)
        }

        fn disable_extension(&self, uuid: &str) -> Result<bool, dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "DisableExtension", (uuid,))
                .map(|r: (bool,)| r.0)
        }

        fn launch_extension_prefs(&self, uuid: &str) -> Result<(), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Extensions",
                "LaunchExtensionPrefs",
                (uuid,),
            )
        }

        fn open_extension_prefs(
            &self,
            uuid: &str,
            parent_window: &str,
            options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
        ) -> Result<(), dbus::Error> {
            self.method_call(
                "org.gnome.Shell.Extensions",
                "OpenExtensionPrefs",
                (uuid, parent_window, options),
            )
        }

        fn check_for_updates(&self) -> Result<(), dbus::Error> {
            self.method_call("org.gnome.Shell.Extensions", "CheckForUpdates", ())
        }

        fn shell_version(&self) -> Result<String, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.gnome.Shell.Extensions",
                "ShellVersion",
            )
        }

        fn user_extensions_enabled(&self) -> Result<bool, dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                self,
                "org.gnome.Shell.Extensions",
                "UserExtensionsEnabled",
            )
        }

        fn set_user_extensions_enabled(&self, value: bool) -> Result<(), dbus::Error> {
            <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
                self,
                "org.gnome.Shell.Extensions",
                "UserExtensionsEnabled",
                value,
            )
        }
    }

    #[derive(Debug)]
    pub struct OrgGnomeShellExtensionsExtensionStateChanged {
        pub uuid: String,
        pub state:
            ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
    }

    impl arg::AppendAll for OrgGnomeShellExtensionsExtensionStateChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.uuid, i);
            arg::RefArg::append(&self.state, i);
        }
    }

    impl arg::ReadAll for OrgGnomeShellExtensionsExtensionStateChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgGnomeShellExtensionsExtensionStateChanged {
                uuid: i.read()?,
                state: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgGnomeShellExtensionsExtensionStateChanged {
        const NAME: &'static str = "ExtensionStateChanged";
        const INTERFACE: &'static str = "org.gnome.Shell.Extensions";
    }

    #[derive(Debug)]
    pub struct OrgGnomeShellExtensionsExtensionStatusChanged {
        pub uuid: String,
        pub state: i32,
        pub error: String,
    }

    impl arg::AppendAll for OrgGnomeShellExtensionsExtensionStatusChanged {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.uuid, i);
            arg::RefArg::append(&self.state, i);
            arg::RefArg::append(&self.error, i);
        }
    }

    impl arg::ReadAll for OrgGnomeShellExtensionsExtensionStatusChanged {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            Ok(OrgGnomeShellExtensionsExtensionStatusChanged {
                uuid: i.read()?,
                state: i.read()?,
                error: i.read()?,
            })
        }
    }

    impl dbus::message::SignalArgs for OrgGnomeShellExtensionsExtensionStatusChanged {
        const NAME: &'static str = "ExtensionStatusChanged";
        const INTERFACE: &'static str = "org.gnome.Shell.Extensions";
    }
}
*/
