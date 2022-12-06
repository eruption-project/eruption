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

use std::{
    io::Read,
    process::{Command, Stdio},
};

use crate::{preferences, util};
use glib::clone;
use gtk::prelude::*;

use crate::constants;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Daemon action failed")]
    ActionFailed,
    // #[error("Unknown error")]
    // UnknownError,
}

/// Initialize page "Profiles"
pub fn initialize_settings_page(builder: &gtk::Builder) -> Result<()> {
    let host_name: gtk::Entry = builder.object("host_name").unwrap();
    let port_number: gtk::SpinButton = builder.object("port_number").unwrap();

    let eruption_daemon_switch: gtk::Switch = builder.object("eruption_daemon_switch").unwrap();
    let process_monitor_daemon_switch: gtk::Switch =
        builder.object("process_monitor_daemon_switch").unwrap();
    let audio_proxy_daemon_switch: gtk::Switch =
        builder.object("audio_proxy_daemon_switch").unwrap();
    let fx_proxy_daemon_switch: gtk::Switch = builder.object("fx_proxy_daemon_switch").unwrap();

    let eruption_daemon_status_label: gtk::Label =
        builder.object("eruption_daemon_status_label").unwrap();
    let process_monitor_daemon_status_label: gtk::Label = builder
        .object("process_monitor_daemon_status_label")
        .unwrap();
    let audio_proxy_daemon_status_label: gtk::Label =
        builder.object("audio_proxy_daemon_status_label").unwrap();
    let fx_proxy_daemon_status_label: gtk::Label =
        builder.object("fx_proxy_daemon_status_label").unwrap();

    let restart_eruption_button: gtk::Button = builder.object("restart_eruption_button").unwrap();
    let restart_process_monitor_button: gtk::Button =
        builder.object("restart_process_monitor_button").unwrap();
    let restart_audio_proxy_button: gtk::Button =
        builder.object("restart_audio_proxy_button").unwrap();
    let restart_fx_proxy_button: gtk::Button = builder.object("restart_fx_proxy_button").unwrap();

    host_name.connect_changed(move |entry| {
        preferences::set_host_name(&entry.text())
            .unwrap_or_else(|e| tracing::error!("Could not save a settings value: {}", e));
    });

    port_number.connect_changed(move |entry| {
        preferences::set_port_number(entry.value() as u16)
            .unwrap_or_else(|e| tracing::error!("Could not save a settings value: {}", e));
    });

    host_name.set_text(&preferences::get_host_name()?);
    port_number.set_value(preferences::get_port_number()? as f64);

    // update daemon status
    eruption_daemon_status_label.set_use_markup(true);
    process_monitor_daemon_status_label.set_use_markup(true);
    audio_proxy_daemon_status_label.set_use_markup(true);
    fx_proxy_daemon_status_label.set_use_markup(true);

    /* eruption_daemon_switch.connect_state_set(move |_sw, enabled| {
        let _status = set_daemon_status(Daemon::Eruption, enabled);

        gtk::Inhibit(false)
    }); */

    process_monitor_daemon_switch.connect_state_set(move |_sw, enabled| {
        let _status = set_daemon_status(Daemon::ProcessMonitor, enabled);

        gtk::Inhibit(false)
    });

    audio_proxy_daemon_switch.connect_state_set(move |_sw, enabled| {
        let _status = set_daemon_status(Daemon::AudioProxy, enabled);

        gtk::Inhibit(false)
    });

    fx_proxy_daemon_switch.connect_state_set(move |_sw, enabled| {
        let _status = set_daemon_status(Daemon::FxProxy, enabled);

        gtk::Inhibit(false)
    });

    restart_eruption_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_eruption_daemon() {
            tracing::error!("Could not restart the Eruption daemon: {e}");
        } else {
            tracing::info!("Successfully restarted the Eruption daemon");
        }
    });

    restart_process_monitor_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_process_monitor_daemon() {
            tracing::error!("Could not restart the Eruption process monitor daemon: {e}");
        } else {
            tracing::info!("Successfully restarted the Eruption process monitor daemon");
        }
    });

    restart_audio_proxy_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_audio_proxy_daemon() {
            tracing::error!("Could not restart the Eruption audio proxy daemon: {e}");
        } else {
            tracing::info!("Successfully restarted the Eruption audio proxy daemon");
        }
    });

    restart_fx_proxy_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_fx_proxy_daemon() {
            tracing::error!("Could not restart the Eruption fx proxy daemon: {e}");
        } else {
            tracing::info!("Successfully restarted the Eruption fx proxy daemon");
        }
    });

    crate::register_timer(
        500,
        clone!(@weak eruption_daemon_status_label, @weak eruption_daemon_switch, @weak process_monitor_daemon_status_label,
                    @weak process_monitor_daemon_switch, @weak audio_proxy_daemon_switch, @weak audio_proxy_daemon_status_label
                    => @default-return Ok(()), move || {

                        match get_daemon_status(Daemon::Eruption) {
                            Ok(status) => {
                                    match status {
                                        ServiceStatus::Active => {
                                            eruption_daemon_status_label.set_label("<b><span background='#00ff00' foreground='white'>    OK    </span></b>");
                                            eruption_daemon_switch.set_state(true);
                                            eruption_daemon_switch.set_sensitive(false);
                                        }

                                        ServiceStatus::Inactive => {
                                            eruption_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>offline</span></b>");
                                            eruption_daemon_switch.set_state(false);
                                            eruption_daemon_switch.set_sensitive(false);
                                        }

                                        ServiceStatus::Failed =>  {
                                            eruption_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>failed</span></b>");
                                            eruption_daemon_switch.set_state(false);
                                            eruption_daemon_switch.set_sensitive(false);
                                        }

                                        ServiceStatus::Unknown =>  {
                                            eruption_daemon_status_label.set_label("<b><span foreground='white'>unknown</span></b>");
                                            eruption_daemon_switch.set_sensitive(false);
                                        }
                                }
                            }

                            Err(_e) => {
                                eruption_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'> error </span></b>");
                                eruption_daemon_switch.set_state(false);
                                eruption_daemon_switch.set_sensitive(false);
                            }
                        }

                        match get_daemon_status(Daemon::ProcessMonitor) {
                            Ok(status) => {
                                    match status {
                                        ServiceStatus::Active => {
                                            process_monitor_daemon_status_label.set_label("<b><span background='#00ff00' foreground='white'>    OK    </span></b>");
                                            process_monitor_daemon_switch.set_state(true);
                                            process_monitor_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Inactive => {
                                            process_monitor_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>offline</span></b>");
                                            process_monitor_daemon_switch.set_state(false);
                                            process_monitor_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Failed =>  {
                                            process_monitor_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>failed</span></b>");
                                            process_monitor_daemon_switch.set_state(false);
                                            process_monitor_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Unknown =>  {
                                            process_monitor_daemon_status_label.set_label("<b><span foreground='white'>unknown</span></b>");
                                            process_monitor_daemon_switch.set_sensitive(false);
                                        }
                                }
                            }

                            Err(_e) => {
                                process_monitor_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>error</span></b>");
                                process_monitor_daemon_switch.set_state(false);
                                process_monitor_daemon_switch.set_sensitive(false);
                            }
                        }

                        match get_daemon_status(Daemon::AudioProxy) {
                            Ok(status) => {
                                    match status {
                                        ServiceStatus::Active => {
                                            audio_proxy_daemon_status_label.set_label("<b><span background='#00ff00' foreground='white'>    OK    </span></b>");
                                            audio_proxy_daemon_switch.set_state(true);
                                            audio_proxy_daemon_switch.set_sensitive(true);

                                        }

                                        ServiceStatus::Inactive => {
                                            audio_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>offline</span></b>");
                                            audio_proxy_daemon_switch.set_state(false);
                                            audio_proxy_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Failed =>  {
                                            audio_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>failed</span></b>");
                                            audio_proxy_daemon_switch.set_state(false);
                                            audio_proxy_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Unknown =>  {
                                            audio_proxy_daemon_status_label.set_label("<b><span foreground='white'>unknown</span></b>");
                                            audio_proxy_daemon_switch.set_sensitive(false);
                                        }
                                }
                            }

                            Err(_e) => {
                                audio_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>error</span></b>");
                                audio_proxy_daemon_switch.set_state(false);
                                audio_proxy_daemon_switch.set_sensitive(false);
                            }
                        }

                        match get_daemon_status(Daemon::FxProxy) {
                            Ok(status) => {
                                    match status {
                                        ServiceStatus::Active => {
                                            fx_proxy_daemon_status_label.set_label("<b><span background='#00ff00' foreground='white'>    OK    </span></b>");
                                            fx_proxy_daemon_switch.set_state(true);
                                            fx_proxy_daemon_switch.set_sensitive(true);

                                        }

                                        ServiceStatus::Inactive => {
                                            fx_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>offline</span></b>");
                                            fx_proxy_daemon_switch.set_state(false);
                                            fx_proxy_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Failed =>  {
                                            fx_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>failed</span></b>");
                                            fx_proxy_daemon_switch.set_state(false);
                                            fx_proxy_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Unknown =>  {
                                            fx_proxy_daemon_status_label.set_label("<b><span foreground='white'>unknown</span></b>");
                                            fx_proxy_daemon_switch.set_sensitive(false);
                                        }
                                }
                            }

                            Err(_e) => {
                                fx_proxy_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>error</span></b>");
                                fx_proxy_daemon_switch.set_state(false);
                                fx_proxy_daemon_switch.set_sensitive(false);
                            }
                        }

            Ok(())
        }),
    )?;

    Ok(())
}

#[derive(Debug)]
enum Daemon {
    Eruption,
    ProcessMonitor,
    AudioProxy,
    FxProxy,
}

fn set_daemon_status(daemon: Daemon, running: bool) -> Result<()> {
    let unit_file = match daemon {
        Daemon::Eruption => constants::UNIT_NAME_ERUPTION,
        Daemon::ProcessMonitor => constants::UNIT_NAME_PROCESS_MONITOR,
        Daemon::AudioProxy => constants::UNIT_NAME_AUDIO_PROXY,
        Daemon::FxProxy => constants::UNIT_NAME_FX_PROXY,
    };

    let user_or_system = match daemon {
        Daemon::Eruption => "--system",
        Daemon::ProcessMonitor => "--user",
        Daemon::AudioProxy => "--user",
        Daemon::FxProxy => "--user",
    };

    let action = if running { "start" } else { "stop" };

    let status = Command::new("/usr/bin/systemctl")
        // .stdout(Stdio::null())
        .arg(user_or_system)
        .arg(action)
        .arg(unit_file)
        .status()?;

    let exit_code = status.code().unwrap_or(0);

    if exit_code != 0 {
        Err(ServiceError::ActionFailed {}.into())
    } else {
        Ok(())
    }
}
pub enum ServiceStatus {
    Unknown,
    Active,
    Inactive,
    Failed,
}

fn get_daemon_status(daemon: Daemon) -> Result<ServiceStatus> {
    let unit_file = match daemon {
        Daemon::Eruption => constants::UNIT_NAME_ERUPTION,
        Daemon::ProcessMonitor => constants::UNIT_NAME_PROCESS_MONITOR,
        Daemon::AudioProxy => constants::UNIT_NAME_AUDIO_PROXY,
        Daemon::FxProxy => constants::UNIT_NAME_FX_PROXY,
    };

    let user_or_system = match daemon {
        Daemon::Eruption => "--system",
        Daemon::ProcessMonitor => "--user",
        Daemon::AudioProxy => "--user",
        Daemon::FxProxy => "--user",
    };

    let mut status = Command::new("/usr/bin/systemctl")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(user_or_system)
        .arg("is-failed")
        .arg(unit_file)
        .spawn()?;

    let _status = status.wait()?;

    match status.stdout {
        Some(ref mut out) => {
            let mut output = String::new();
            out.read_to_string(&mut output)?;

            match output.trim() {
                "failed" => Ok(ServiceStatus::Failed),
                "active" => Ok(ServiceStatus::Active),
                "inactive" => Ok(ServiceStatus::Inactive),

                _ => Ok(ServiceStatus::Unknown),
            }
        }

        None => Err(ServiceError::ActionFailed {}.into()),
    }
}
