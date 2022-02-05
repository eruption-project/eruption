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

use crate::canvas::Canvas;
use crate::transport::{ServerStatus, Transport};
use crate::{util, Result};
use eyre::eyre;
use parking_lot::Mutex;
use prost::Message;
use protocol::request::Payload as RequestPayload;
use protocol::response::Payload as ResponsePayload;
use socket2::{Domain, SockAddr, Socket, Type};
use std::io::{Cursor, Write};
use std::mem::MaybeUninit;
use std::sync::Arc;

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/sdk_support.rs"));
}

const SOCKET_ADDRESS: &str = "/run/eruption/control.sock";
const MAX_BUF: usize = 4096;

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
        let addr = SockAddr::unix(&SOCKET_ADDRESS)?;
        self.socket.lock().connect(&addr)?;

        Ok(())
    }

    fn disconnect(&mut self) -> Result<()> {
        self.socket.lock().flush()?;
        // self.socket.lock().shutdown(Shutdown::Both)?;

        Ok(())
    }

    fn get_server_status(&self) -> Result<ServerStatus> {
        let mut request = protocol::Request::default();
        request.set_request_type(protocol::RequestType::Status);

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
                        let ResponsePayload::Data(payload) = result.payload.unwrap();

                        Ok(ServerStatus {
                            server: String::from_utf8_lossy(&payload).to_string(),
                        })
                    }

                    Err(_e) => Err(eyre!("Lost connection to Eruption")),
                }
            }

            Err(_e) => Err(eyre!("Lost connection to Eruption")),
        }
    }

    fn submit_canvas(&self, canvas: &Canvas) -> Result<()> {
        let mut request = protocol::Request::default();
        request.set_request_type(protocol::RequestType::SetCanvas);

        let bytes: Vec<u8> = canvas
            .data
            .iter()
            .flat_map(|c| vec![c.r(), c.g(), c.b(), c.a()])
            .collect();

        request.payload = Some(RequestPayload::Data(bytes));

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
