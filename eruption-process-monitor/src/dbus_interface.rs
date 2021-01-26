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
*/

use crossbeam::channel::Sender;
use dbus::{ffidisp::Connection, ffidisp::NameFlag, MethodErr};
use dbus_tree::{Factory, Signal};
use indexmap::IndexMap;
use log::*;
use std::sync::Arc;

use crate::{Action, RuleMetadata, Selector, WindowFocusedSelectorMode};

/// D-Bus messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum DbusApiError {
    #[error("D-Bus not connected")]
    BusNotConnected {},

    #[error("Invalid argument")]
    InvalidArgument {},
}

/// D-Bus API support
pub struct DbusApi {
    connection: Option<Arc<Connection>>,

    rules_changed: Arc<Signal<()>>,
}

#[allow(dead_code)]
impl DbusApi {
    /// Initialize the D-Bus API
    pub fn new(_dbus_tx: Sender<Message>) -> Result<Self> {
        // let dbus_tx_clone = dbus_tx.clone();

        let c = Connection::new_session()?;
        c.register_name(
            "org.eruption.process_monitor",
            NameFlag::ReplaceExisting as u32,
        )?;

        let c_clone = Arc::new(c);
        let f = Factory::new_fn::<()>();

        let rules_changed_signal =
            Arc::new(
                f.signal("RulesChanged", ())
                    .sarg::<Vec<(String, String, String, String)>, _>("rules"),
            );
        let rules_changed_signal_clone = rules_changed_signal.clone();

        let tree = f.tree(()).add(
            f.object_path("/rules", ()).introspectable().add(
                f.interface("org.eruption.process_monitor.Rules", ())
                    .add_m(
                        f.method("EnumRules", (), move |m| {
                            let rules_map = crate::RULES_MAP.read();
                            let s = rules_map
                                .iter()
                                .map(|(selector, (metadata, action))| {
                                    let (sensor_val, selector_val) = match selector {
                                        Selector::ProcessExec { comm } => {
                                            ("exec".to_string(), comm)
                                        }

                                        Selector::WindowFocused { mode, regex } => match mode {
                                            WindowFocusedSelectorMode::WindowName => {
                                                ("window-name".to_string(), regex)
                                            }
                                            WindowFocusedSelectorMode::WindowInstance => {
                                                ("window-instance".to_string(), regex)
                                            }
                                            WindowFocusedSelectorMode::WindowClass => {
                                                ("window-class".to_string(), regex)
                                            }
                                        },
                                    };

                                    let action_val = match action {
                                        Action::SwitchToProfile { profile_name } => {
                                            format!("{}", profile_name)
                                        }
                                        Action::SwitchToSlot { slot_index } => {
                                            format!("{}", slot_index)
                                        }
                                    };

                                    let mut metadata_val = String::new();
                                    if metadata.enabled {
                                        metadata_val.push_str("enabled");
                                    } else {
                                        metadata_val.push_str("disabled");
                                    }

                                    if metadata.internal {
                                        metadata_val.push_str(",internal");
                                    } else {
                                        metadata_val.push_str(",user-defined");
                                    }

                                    (sensor_val, selector_val, action_val, metadata_val)
                                })
                                .collect::<Vec<_>>();
                            Ok(vec![m.msg.method_return().append1(s)])
                        })
                        .outarg::<Vec<(String, String, String, String)>, _>("rules"),
                    )
                    .add_m(
                        f.method("SetRules", (), move |m| {
                            let mut rules_map = IndexMap::new();

                            let rules: Vec<(String, String, String, String)> = m.msg.read1()?;

                            for rule in rules {
                                let sensor_val = rule.0;
                                let selector_val = rule.1;
                                let action_val = rule.2;
                                let metadata_val = rule.3;

                                fn parse_rule(
                                    sensor_val: &str,
                                    selector_val: &str,
                                    action_val: &str,
                                    metadata_val: &str,
                                ) -> Result<(Selector, (RuleMetadata, Action))>
                                {
                                    let sensor;
                                    let metadata;
                                    let action;

                                    match sensor_val {
                                        "exec" => {
                                            sensor = Selector::ProcessExec {
                                                comm: selector_val.into(),
                                            }
                                        }

                                        "window-name" => {
                                            sensor = Selector::WindowFocused {
                                                mode: WindowFocusedSelectorMode::WindowName,
                                                regex: selector_val.into(),
                                            }
                                        }

                                        "window-instance" => {
                                            sensor = Selector::WindowFocused {
                                                mode: WindowFocusedSelectorMode::WindowInstance,
                                                regex: selector_val.into(),
                                            }
                                        }

                                        "window-class" => {
                                            sensor = Selector::WindowFocused {
                                                mode: WindowFocusedSelectorMode::WindowClass,
                                                regex: selector_val.into(),
                                            }
                                        }

                                        _ => return Err(DbusApiError::InvalidArgument {}.into()),
                                    }

                                    let enabled = metadata_val.contains("enabled");
                                    let internal = metadata_val.contains("internal");

                                    metadata = RuleMetadata { enabled, internal };

                                    if action_val.contains(".profile") {
                                        action = Action::SwitchToProfile {
                                            profile_name: action_val.to_string(),
                                        };
                                    } else {
                                        action = Action::SwitchToSlot {
                                            slot_index: action_val.parse::<u64>()?,
                                        };
                                    }

                                    Ok((sensor, (metadata, action)))
                                }

                                let (selector, (metadata, action)) = parse_rule(
                                    &sensor_val,
                                    &selector_val,
                                    &action_val,
                                    &metadata_val,
                                )
                                .map_err(|_e| MethodErr::invalid_arg("rules"))?;

                                rules_map.insert(selector, (metadata, action));
                            }

                            *crate::RULES_MAP.write() = rules_map;

                            crate::save_rules_map().map_err(|_e| {
                                dbus::Error::new_failed("Could not save the rules map")
                            })?;

                            Ok(vec![m.msg.method_return()])
                        })
                        .inarg::<Vec<(String, String, String, String)>, _>("rules"),
                    )
                    .add_s(rules_changed_signal_clone),
            ),
        );

        tree.set_registered(&*c_clone, true)
            .unwrap_or_else(|e| error!("Could not register the tree: {}", e));
        c_clone.add_handler(tree);

        Ok(Self {
            connection: Some(c_clone),
            rules_changed: rules_changed_signal,
        })
    }

    pub fn notify_rules_changed(&self) {
        let rules_map = crate::RULES_MAP.read();
        let s = rules_map
            .iter()
            .map(|(selector, (metadata, action))| {
                let (sensor_val, selector_val) = match selector {
                    Selector::ProcessExec { comm } => ("exec".to_string(), comm),

                    Selector::WindowFocused { mode, regex } => match mode {
                        WindowFocusedSelectorMode::WindowName => ("window-name".to_string(), regex),
                        WindowFocusedSelectorMode::WindowInstance => {
                            ("window-instance".to_string(), regex)
                        }
                        WindowFocusedSelectorMode::WindowClass => {
                            ("window-class".to_string(), regex)
                        }
                    },
                };

                let action_val = match action {
                    Action::SwitchToProfile { profile_name } => {
                        format!("{}", profile_name)
                    }
                    Action::SwitchToSlot { slot_index } => {
                        format!("{}", slot_index)
                    }
                };

                let mut metadata_val = String::new();
                if metadata.enabled {
                    metadata_val.push_str("enabled");
                } else {
                    metadata_val.push_str("disabled");
                }

                if metadata.internal {
                    metadata_val.push_str(",internal");
                } else {
                    metadata_val.push_str(",user-defined");
                }

                (sensor_val, selector_val, action_val, metadata_val)
            })
            .collect::<Vec<_>>();

        self.connection
            .as_ref()
            .unwrap()
            .send(self.rules_changed.emit(
                &"/rules".into(),
                &"org.eruption.process_monitor.Rules".into(),
                &s,
            ))
            .unwrap();
    }

    /// Returns true if an event is pending on the D-Bus connection
    pub fn has_pending_event(&self) -> Result<bool> {
        match self.connection {
            Some(ref connection) => {
                let count = connection.incoming(0).peekable().count();

                if count > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }

    /// Get the next event from D-Bus
    pub fn get_next_event(&self) -> Result<()> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(0).next() {
                    // For the actual event handler code please see
                    // implementation of `struct DbusApi`
                    debug!("Message: {:?}", item);
                } else {
                    trace!("Received a timeout message");
                }

                Ok(())
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }

    pub fn get_next_event_timeout(&self, timeout_ms: u32) -> Result<()> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(timeout_ms).next() {
                    // For the actual event handler code please see
                    // implementation of `struct DbusApi`
                    debug!("Message: {:?}", item);
                } else {
                    trace!("Received a timeout message");
                }

                Ok(())
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }
}

/// Initialize the D-Bus API
pub fn initialize(dbus_tx: Sender<Message>) -> Result<DbusApi> {
    DbusApi::new(dbus_tx)
}

#[allow(dead_code)]
mod perms {
    use dbus::{arg::RefArg, arg::Variant, blocking::Connection};
    use std::{collections::HashMap, time::Duration};

    use crate::constants;

    pub type Result<T> = std::result::Result<T, eyre::Error>;

    pub fn has_monitor_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.process_monitor.monitor",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    pub fn has_settings_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.process_monitor.settings",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    pub fn has_manage_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.process_monitor.manage",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    mod bus {
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
            fn get_connection_selinux_security_context(
                &self,
                arg0: &str,
            ) -> Result<Vec<u8>, dbus::Error>;
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
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RequestName", (arg0, arg1))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReleaseName", (arg0,))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "StartServiceByName", (arg0, arg1))
                    .and_then(|r: (u32,)| Ok(r.0))
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
                    .and_then(|r: (bool,)| Ok(r.0))
            }

            fn list_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListNames", ())
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListActivatableNames", ())
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn add_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "AddMatch", (arg0,))
            }

            fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RemoveMatch", (arg0,))
            }

            fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetNameOwner", (arg0,))
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListQueuedOwners", (arg0,))
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetConnectionUnixUser", (arg0,))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "GetConnectionUnixProcessID",
                    (arg0,),
                )
                .and_then(|r: (u32,)| Ok(r.0))
            }

            fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetAdtAuditSessionData", (arg0,))
                    .and_then(|r: (Vec<u8>,)| Ok(r.0))
            }

            fn get_connection_selinux_security_context(
                &self,
                arg0: &str,
            ) -> Result<Vec<u8>, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "GetConnectionSELinuxSecurityContext",
                    (arg0,),
                )
                .and_then(|r: (Vec<u8>,)| Ok(r.0))
            }

            fn reload_config(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReloadConfig", ())
            }

            fn get_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetId", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn get_connection_credentials(
                &self,
                arg0: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            > {
                self.method_call("org.freedesktop.DBus", "GetConnectionCredentials", (arg0,))
                    .and_then(
                        |r: (
                            ::std::collections::HashMap<
                                String,
                                arg::Variant<Box<dyn arg::RefArg + 'static>>,
                            >,
                        )| Ok(r.0),
                    )
            }

            fn features(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.DBus",
                    "Features",
                )
            }

            fn interfaces(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
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

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
        {
            fn introspect(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                    .and_then(|r: (String,)| Ok(r.0))
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
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn ping(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
            }
        }
    }

    mod polkit {
        // This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.PolicyKit1 -p /org/freedesktop/PolicyKit1/Authority -m None`, see https://github.com/diwic/dbus-rs
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
                .and_then(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| Ok(r.0))
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
                .and_then(
                    |r: (
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    )| Ok(r.0),
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

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
        {
            fn introspect(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }
        }

        pub trait OrgFreedesktopDBusPeer {
            fn ping(&self) -> Result<(), dbus::Error>;
            fn get_machine_id(&self) -> Result<String, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusPeer
            for blocking::Proxy<'a, C>
        {
            fn ping(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
            }

            fn get_machine_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }
        }

        pub trait OrgFreedesktopPolicyKit1Authority {
            fn enumerate_actions(
                &self,
                locale: &str,
            ) -> Result<
                Vec<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    u32,
                    u32,
                    u32,
                    ::std::collections::HashMap<String, String>,
                )>,
                dbus::Error,
            >;
            fn check_authorization(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                action_id: &str,
                details: ::std::collections::HashMap<&str, &str>,
                flags: u32,
                cancellation_id: &str,
            ) -> Result<(bool, bool, ::std::collections::HashMap<String, String>), dbus::Error>;
            fn cancel_check_authorization(&self, cancellation_id: &str) -> Result<(), dbus::Error>;
            fn register_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
            ) -> Result<(), dbus::Error>;
            fn register_authentication_agent_with_options(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
                options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
            ) -> Result<(), dbus::Error>;
            fn unregister_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                object_path: &str,
            ) -> Result<(), dbus::Error>;
            fn authentication_agent_response(
                &self,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn authentication_agent_response2(
                &self,
                uid: u32,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn enumerate_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<
                Vec<(
                    String,
                    String,
                    (
                        String,
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    ),
                    u64,
                    u64,
                )>,
                dbus::Error,
            >;
            fn revoke_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn revoke_temporary_authorization_by_id(&self, id: &str) -> Result<(), dbus::Error>;
            fn backend_name(&self) -> Result<String, dbus::Error>;
            fn backend_version(&self) -> Result<String, dbus::Error>;
            fn backend_features(&self) -> Result<u32, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopPolicyKit1Authority for blocking::Proxy<'a, C>
        {
            fn enumerate_actions(
                &self,
                locale: &str,
            ) -> Result<
                Vec<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    u32,
                    u32,
                    u32,
                    ::std::collections::HashMap<String, String>,
                )>,
                dbus::Error,
            > {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "EnumerateActions",
                    (locale,),
                )
                .and_then(
                    |r: (
                        Vec<(
                            String,
                            String,
                            String,
                            String,
                            String,
                            String,
                            u32,
                            u32,
                            u32,
                            ::std::collections::HashMap<String, String>,
                        )>,
                    )| Ok(r.0),
                )
            }

            fn check_authorization(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                action_id: &str,
                details: ::std::collections::HashMap<&str, &str>,
                flags: u32,
                cancellation_id: &str,
            ) -> Result<(bool, bool, ::std::collections::HashMap<String, String>), dbus::Error>
            {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "CheckAuthorization",
                    (subject, action_id, details, flags, cancellation_id),
                )
                .and_then(|r: ((bool, bool, ::std::collections::HashMap<String, String>),)| Ok(r.0))
            }

            fn cancel_check_authorization(&self, cancellation_id: &str) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "CancelCheckAuthorization",
                    (cancellation_id,),
                )
            }

            fn register_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RegisterAuthenticationAgent",
                    (subject, locale, object_path),
                )
            }

            fn register_authentication_agent_with_options(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
                options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RegisterAuthenticationAgentWithOptions",
                    (subject, locale, object_path, options),
                )
            }

            fn unregister_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                object_path: &str,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "UnregisterAuthenticationAgent",
                    (subject, object_path),
                )
            }

            fn authentication_agent_response(
                &self,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "AuthenticationAgentResponse",
                    (cookie, identity),
                )
            }

            fn authentication_agent_response2(
                &self,
                uid: u32,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "AuthenticationAgentResponse2",
                    (uid, cookie, identity),
                )
            }

            fn enumerate_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<
                Vec<(
                    String,
                    String,
                    (
                        String,
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    ),
                    u64,
                    u64,
                )>,
                dbus::Error,
            > {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "EnumerateTemporaryAuthorizations",
                    (subject,),
                )
                .and_then(
                    |r: (
                        Vec<(
                            String,
                            String,
                            (
                                String,
                                ::std::collections::HashMap<
                                    String,
                                    arg::Variant<Box<dyn arg::RefArg + 'static>>,
                                >,
                            ),
                            u64,
                            u64,
                        )>,
                    )| Ok(r.0),
                )
            }

            fn revoke_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RevokeTemporaryAuthorizations",
                    (subject,),
                )
            }

            fn revoke_temporary_authorization_by_id(&self, id: &str) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RevokeTemporaryAuthorizationById",
                    (id,),
                )
            }

            fn backend_name(&self) -> Result<String, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendName",
                )
            }

            fn backend_version(&self) -> Result<String, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendVersion",
                )
            }

            fn backend_features(&self) -> Result<u32, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendFeatures",
                )
            }
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopPolicyKit1AuthorityChanged {}

        impl arg::AppendAll for OrgFreedesktopPolicyKit1AuthorityChanged {
            fn append(&self, _: &mut arg::IterAppend) {}
        }

        impl arg::ReadAll for OrgFreedesktopPolicyKit1AuthorityChanged {
            fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopPolicyKit1AuthorityChanged {})
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopPolicyKit1AuthorityChanged {
            const NAME: &'static str = "Changed";
            const INTERFACE: &'static str = "org.freedesktop.PolicyKit1.Authority";
        }
    }
}
