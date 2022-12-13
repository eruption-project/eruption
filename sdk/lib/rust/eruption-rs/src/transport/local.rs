/*  SPDX-License-Identifier: LGPL-3.0-or-later  */

/*
    This file is part of the Eruption SDK.

    The Eruption SDK is free software: you can redistribute it and/or modify
    it under the terms of the GNU Lesser General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    The Eruption SDK is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with the Eruption SDK.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use crate::canvas::Canvas;
use crate::hardware::HotplugInfo;
use crate::transport::{ServerStatus, Transport};
use crate::{util, Result};
use eyre::eyre;
use parking_lot::Mutex;
use prost::Message;
use socket2::{Domain, SockAddr, Socket, Type};
use std::collections::HashMap;
use std::io::{Cursor, Write};
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/sdk_support.rs"));
}

const SOCKET_ADDRESS: &str = "/run/eruption/control.sock";
const MAX_BUF: usize = 4096 * 16;

#[derive(Debug, Clone)]
pub struct LocalTransport {
    pub(crate) socket: Arc<Mutex<Socket>>,
}

impl LocalTransport {
    pub fn new() -> Result<Self> {
        Ok(Self {
            socket: Arc::new(Mutex::new(Socket::new(
                Domain::UNIX,
                Type::SEQPACKET,
                None,
            )?)),
        })
    }
}

impl Transport for LocalTransport {
    fn connect(&mut self) -> Result<()> {
        let addr = SockAddr::unix(SOCKET_ADDRESS)?;
        self.socket.lock().connect(&addr)?;

        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.socket.lock().flush()?;
        // self.socket.lock().shutdown(Shutdown::Both)?;

        Ok(())
    }

    fn get_server_status(&self) -> Result<ServerStatus> {
        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::Status(
                protocol::StatusRequest {},
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;
                        if let Some(protocol::response::ResponseMessage::Status(status_response)) =
                            result.response_message
                        {
                            Ok(ServerStatus {
                                server: status_response.description,
                            })
                        } else {
                            Err(eyre!("Unexpected response"))
                        }
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn get_active_profile(&self) -> Result<PathBuf> {
        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::ActiveProfile(
                protocol::ActiveProfileRequest {},
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;
                        if let Some(protocol::response::ResponseMessage::ActiveProfile(
                            active_profile_response,
                        )) = result.response_message
                        {
                            Ok(PathBuf::from(&active_profile_response.profile_file))
                        } else {
                            Err(eyre!("Unexpected response"))
                        }
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn switch_profile(&self, profile_file: &Path) -> Result<bool> {
        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::SwitchProfile(
                protocol::SwitchProfileRequest {
                    profile_file: profile_file.to_string_lossy().to_string(),
                },
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;
                        if let Some(protocol::response::ResponseMessage::SwitchProfile(
                            switch_profile_response,
                        )) = result.response_message
                        {
                            Ok(switch_profile_response.switched)
                        } else {
                            Err(eyre!("Unexpected response"))
                        }
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn set_parameters(
        &self,
        profile_file: &Path,
        script_file: &Path,
        parameter_values: HashMap<String, String>,
    ) -> Result<()> {
        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::SetParameters(
                protocol::SetParametersRequest {
                    profile_file: profile_file.to_string_lossy().to_string(),
                    script_file: script_file.to_string_lossy().to_string(),
                    parameter_values,
                },
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;
                        if let Some(protocol::response::ResponseMessage::SetParameters(
                            _set_parameters_response,
                        )) = result.response_message
                        {
                            Ok(())
                        } else {
                            Err(eyre!("Unexpected response"))
                        }
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn submit_canvas(&self, canvas: &Canvas) -> Result<()> {
        let bytes: Vec<u8> = canvas
            .data
            .iter()
            .flat_map(|c| vec![c.r(), c.g(), c.b(), c.a()])
            .collect();

        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::SetCanvas(
                protocol::SetCanvasRequest { canvas: bytes },
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;
                        if let Some(protocol::response::ResponseMessage::SetCanvas(
                            _set_canvas_response,
                        )) = result.response_message
                        {
                            Ok(())
                        } else {
                            Err(eyre!("Unexpected response"))
                        }
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn notify_device_hotplug(&self, hotplug_info: &HotplugInfo) -> Result<()> {
        let config = bincode::config::standard();
        let bytes: Vec<u8> = bincode::encode_to_vec(hotplug_info, config).unwrap();

        let request = protocol::Request {
            request_message: Some(protocol::request::RequestMessage::NotifyHotplug(
                protocol::NotifyHotplugRequest { payload: bytes },
            )),
        };

        let mut buf = Vec::new();
        request.encode_length_delimited(&mut buf)?;

        // send data
        let socket = self.socket.lock();
        match socket.send(&buf) {
            Ok(_n) => {
                // read response
                let mut tmp = [MaybeUninit::zeroed(); MAX_BUF];

                match socket.recv(&mut tmp) {
                    Ok(0) => Err(eyre!("Lost connection to Eruption")),

                    Ok(_n) => {
                        let tmp = unsafe { util::assume_init(&tmp[..tmp.len()]) };
                        let _result =
                            protocol::Response::decode_length_delimited(&mut Cursor::new(&tmp))?;

                        Ok(())
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }
}

impl Drop for LocalTransport {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
