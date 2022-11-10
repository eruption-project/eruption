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

use super::{Sensor, SensorConfiguration, SENSORS_CONFIGURATION};
use crate::procmon::{self, ProcMon};
use crate::{util, SystemEvent};
use async_trait::async_trait;
use flume::Sender;
use lazy_static::lazy_static;
use log::*;
use std::sync::atomic::AtomicBool;
use std::{sync::atomic::Ordering, thread};

use crate::QUIT;

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// "Process sensor has failed" flag, most likely triggered by a
    /// socket error on the Linux kernel netlink socket
    pub static ref PROCESS_SENSOR_FAILED: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum ProcessSensorError {
    #[error("Socket error")]
    SocketError,

    #[error("Operation not supported")]
    NotSupported,
}

#[derive(Debug, Clone)]
pub struct ProcessSensorData {
    pub comm: String,
    pub pid: i32,
}

impl super::SensorData for ProcessSensorData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct ProcessSensor {
    tx: Option<Sender<SystemEvent>>,
}

impl ProcessSensor {
    pub fn new() -> Self {
        ProcessSensor { tx: None }
    }

    pub fn set_tx(&mut self, tx: &Sender<SystemEvent>) {
        self.tx = Some(tx.clone());
    }

    pub fn spawn_system_monitor_thread(&mut self, sysevents_tx: Sender<SystemEvent>) -> Result<()> {
        self.set_tx(&sysevents_tx);

        thread::Builder::new()
            .name("monitor".to_owned())
            .spawn(move || -> Result<()> {
                let procmon = ProcMon::new()?;

                loop {
                    // process procmon events
                    let event = procmon.wait_for_event();

                    // check if we shall terminate the thread
                    if QUIT.load(Ordering::SeqCst) {
                        break Ok(());
                    }

                    match event.event_type {
                        procmon::EventType::Exec => {
                            let pid = event.pid;

                            sysevents_tx
                                .send(SystemEvent::ProcessExec {
                                    event,
                                    file_name: util::get_process_file_name(pid).ok(),
                                    comm: util::get_process_comm(pid).ok(),
                                })
                                .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
                        }

                        procmon::EventType::Exit => {
                            sysevents_tx
                                .send(SystemEvent::ProcessExit { event })
                                .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
                        }

                        procmon::EventType::SocketError => {
                            log::error!("Error while receiving from Linux kernel netlink socket");

                            PROCESS_SENSOR_FAILED.store(true, Ordering::SeqCst);
                            break Err(ProcessSensorError::SocketError {}.into());
                        }

                        _ => { /* ignore others */ }
                    }
                }
            })?;

        Ok(())
    }
}

#[async_trait]
impl Sensor for ProcessSensor {
    fn get_id(&self) -> String {
        "process".to_string()
    }

    fn get_name(&self) -> String {
        "Process".to_string()
    }

    fn get_description(&self) -> String {
        "Watches the system for process events".to_string()
    }

    fn get_usage_example(&self) -> String {
        r#"
Process:
rules add exec <comm> [<profile-name.profile>|<slot number>]

rules add exec gnome-calc.* /var/lib/eruption/profiles/profile1.profile
rules add exec gnome-calc.* 2
"#
        .to_string()
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        SENSORS_CONFIGURATION
            .read()
            .contains(&SensorConfiguration::EnableProcmon)
    }

    fn is_pollable(&self) -> bool {
        false
    }

    fn is_failed(&self) -> bool {
        PROCESS_SENSOR_FAILED.load(Ordering::SeqCst)
    }

    fn set_failed(&mut self, _failed: bool) {
        // no op
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        Err(ProcessSensorError::NotSupported.into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
