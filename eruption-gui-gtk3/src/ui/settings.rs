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

use crate::{
    events, notifications, preferences,
    timers::{self, TimerMode},
    util::{self, get_daemon_status, set_daemon_status, Daemon, ServiceStatus},
};
use glib::clone;
use gtk::prelude::*;

use super::Pages;

// use crate::constants;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Initialize page "Settings"
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

    eruption_daemon_switch.connect_state_set(move |_sw, enabled| {
        if !(events::shall_ignore_pending_ui_event() && events::shall_ignore_pending_ui_event()) {
            if let Err(e) = set_daemon_status(Daemon::Eruption, enabled) {
                tracing::error!("Operation failed: {}", e);
                notifications::error(&format!("Operation failed: {}", e));
            } else {
                match enabled {
                    true => {
                        notifications::info("Successfully started the Eruption daemon");
                    }
                    false => {
                        notifications::warn("Stopped the Eruption daemon");
                    }
                }
            }

            false.into()
        } else {
            true.into()
        }
    });

    process_monitor_daemon_switch.connect_state_set(move |_sw, enabled| {
        if !(events::shall_ignore_pending_ui_event() && events::shall_ignore_pending_ui_event()) {
            if let Err(e) = set_daemon_status(Daemon::ProcessMonitor, enabled) {
                tracing::error!("Operation failed: {}", e);
                notifications::error(&format!("Operation failed: {}", e));
            } else {
                match enabled {
                    true => {
                        notifications::info(
                            "Successfully started the Eruption process monitor daemon",
                        );
                    }
                    false => {
                        notifications::warn("Stopped the Eruption process monitor daemon");
                    }
                }
            }

            false.into()
        } else {
            true.into()
        }
    });

    audio_proxy_daemon_switch.connect_state_set(move |_sw, enabled| {
        if !(events::shall_ignore_pending_ui_event() && events::shall_ignore_pending_ui_event()) {
            if let Err(e) = set_daemon_status(Daemon::AudioProxy, enabled) {
                tracing::error!("Operation failed: {}", e);
                notifications::error(&format!("Operation failed: {}", e));
            } else {
                match enabled {
                    true => {
                        notifications::info("Successfully started the Eruption audio proxy daemon");
                    }
                    false => {
                        notifications::warn("Stopped the Eruption audio proxy daemon");
                    }
                }
            }

            false.into()
        } else {
            true.into()
        }
    });

    fx_proxy_daemon_switch.connect_state_set(move |_sw, enabled| {
        if !(events::shall_ignore_pending_ui_event() && events::shall_ignore_pending_ui_event()) {
            if let Err(e) = set_daemon_status(Daemon::FxProxy, enabled) {
                tracing::error!("Operation failed: {}", e);
                notifications::error(&format!("Operation failed: {}", e));
            } else {
                match enabled {
                    true => {
                        notifications::info("Successfully started the Eruption fx proxy daemon");
                    }
                    false => {
                        notifications::warn("Stopped the Eruption fx proxy daemon");
                    }
                }
            }

            false.into()
        } else {
            true.into()
        }
    });

    restart_eruption_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_eruption_daemon() {
            tracing::error!("Could not restart the Eruption daemon: {e}");
            notifications::error(&format!("Could not restart the Eruption daemon: {e}"));
        } else {
            tracing::info!("Successfully restarted the Eruption daemon");
            notifications::info("Successfully restarted the Eruption daemon");
        }
    });

    restart_process_monitor_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_process_monitor_daemon() {
            tracing::error!("Could not restart the Eruption process monitor daemon: {e}");
            notifications::error(&format!(
                "Could not restart the Eruption process monitor daemon: {e}"
            ));
        } else {
            tracing::info!("Successfully restarted the Eruption process monitor daemon");
            notifications::info("Successfully restarted the Eruption process monitor daemon");
        }
    });

    restart_audio_proxy_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_audio_proxy_daemon() {
            tracing::error!("Could not restart the Eruption audio proxy daemon: {e}");
            notifications::error(&format!(
                "Could not restart the Eruption audio proxy daemon: {e}"
            ));
        } else {
            tracing::info!("Successfully restarted the Eruption audio proxy daemon");
            notifications::info("Successfully restarted the Eruption audio proxy daemon");
        }
    });

    restart_fx_proxy_button.connect_clicked(|_btn| {
        if let Err(e) = util::restart_fx_proxy_daemon() {
            tracing::error!("Could not restart the Eruption fx proxy daemon: {e}");
            notifications::error(&format!(
                "Could not restart the Eruption fx proxy daemon: {e}"
            ));
        } else {
            tracing::info!("Successfully restarted the Eruption fx proxy daemon");
            notifications::info("Successfully restarted the Eruption fx proxy daemon");
        }
    });

    timers::register_timer(
        timers::SETTINGS_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::Settings as u8),
        500,
        clone!(@weak eruption_daemon_status_label, @weak eruption_daemon_switch, @weak process_monitor_daemon_status_label,
                    @weak process_monitor_daemon_switch, @weak audio_proxy_daemon_switch, @weak audio_proxy_daemon_status_label
                    => @default-return Ok(()), move || {
                        events::ignore_next_ui_events(1);
                        events::ignore_next_dbus_events(1);

                        match get_daemon_status(Daemon::Eruption) {
                            Ok(status) => {
                                    match status {
                                        ServiceStatus::Active => {
                                            eruption_daemon_status_label.set_label("<b><span background='#00ff00' foreground='white'>    OK    </span></b>");
                                            eruption_daemon_switch.set_state(true);
                                            eruption_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Inactive => {
                                            eruption_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>offline</span></b>");
                                            eruption_daemon_switch.set_state(false);
                                            eruption_daemon_switch.set_sensitive(true);
                                        }

                                        ServiceStatus::Failed =>  {
                                            eruption_daemon_status_label.set_label("<b><span background='#ff0000' foreground='white'>failed</span></b>");
                                            eruption_daemon_switch.set_state(false);
                                            eruption_daemon_switch.set_sensitive(true);
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

                        events::reenable_dbus_events();
                        events::reenable_ui_events();

            Ok(())
        }),
    )?;

    Ok(())
}
