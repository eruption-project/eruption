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

use crate::scripting::script::LAST_RENDERED_LED_MAP;
use crate::util::ratelimited;

use crate::{constants, hwdevices, script, SwitchProfileResult};

use lazy_static::lazy_static;
use mlua::prelude::*;

#[cfg(not(target_os = "windows"))]
use nix::poll::{poll, PollFd, PollFlags};
#[cfg(not(target_os = "windows"))]
use nix::unistd::unlink;

use prost::Message;
use socket2::{Domain, SockAddr, Socket, Type};
use std::any::Any;
use std::io::Cursor;
use std::mem::MaybeUninit;
use tracing_mutex::stdsync::RwLock;

#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_os = "windows")]
use std::path::Path;
#[cfg(target_os = "windows")]
use windows_named_pipe::*;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{fs, thread};
use tracing::{debug, error, info, trace, warn};

use crate::{
    hwdevices::RGBA,
    plugins::{self, Plugin},
    scripting::parameters,
    scripting::parameters_util,
};

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/sdk_support.rs"));
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum SdkPluginError {
    #[error("Eruption SDK plugin error: {description}")]
    PluginError { description: String },
}

lazy_static! {
    /// Global LED map, the "canvas"
    pub static ref LED_MAP: Arc<RwLock<Vec<RGBA>>> = Arc::new(RwLock::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; constants::CANVAS_SIZE]));

    /// Frame generation counter, used to detect if an SDK client updated the color map
    pub static ref FRAME_GENERATION_COUNTER_ERUPTION_SDK: AtomicUsize = AtomicUsize::new(0);
}

#[cfg(not(target_os = "windows"))]
lazy_static! {
    pub static ref LISTENER: Arc<RwLock<Option<Socket>>> = Arc::new(RwLock::new(None));
}

#[cfg(target_os = "windows")]
lazy_static! {
    pub static ref LISTENER: Arc<RwLock<Option<PipeListener<'static>>>> =
        Arc::new(RwLock::new(None));
}

use crate::threads::DbusApiEvent;
use bincode::{Decode, Encode};

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct HotplugInfo {
    pub devpath: Option<PathBuf>,
    pub usb_vid: u16,
    pub usb_pid: u16,
}

#[cfg(target_os = "windows")]
pub fn claim_hotplugged_devices(_hotplug_info: &HotplugInfo) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn claim_hotplugged_device(hotplug_info: &HotplugInfo) -> Result<()> {
    use std::ops::RangeFull;

    use indexmap::IndexMap;

    use crate::{hwdevices::DeviceHandle, state};

    if crate::QUIT.load(Ordering::SeqCst) {
        info!("Ignoring device hotplug event since Eruption is shutting down");
    } else {
        // enumerate devices
        info!("Enumerating connected devices...");

        let mut connected_devices = IndexMap::new();

        if let Ok(devices) = hwdevices::probe_devices() {
            if let Some(probed_device) = devices.iter().find(|device| {
                device.read().unwrap().get_dev_paths().contains(
                    &hotplug_info
                        .devpath
                        .clone()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                )
            }) {
                // we found the hot-plug candidate device
                let index = crate::DEVICES.read().unwrap().len();
                let handle = DeviceHandle::from(index as u64);

                info!("Initializing the plugged device...");

                let device = &mut **probed_device.write().unwrap();

                crate::initialize_device(handle, device)?;
                info!("Device initialized successfully");

                let (usb_vid, usb_pid) = (device.get_usb_vid(), device.get_usb_pid());

                // load and initialize global runtime state (late init)
                info!("Loading saved device state...");
                state::init_runtime_state(device)
                    .unwrap_or_else(|e| warn!("Could not parse state file: {}", e));

                connected_devices.insert(handle, probed_device.clone());

                debug!("Sending device hotplug notification...");

                let dbus_api_tx = crate::DBUS_API_TX.read().unwrap();
                let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

                dbus_api_tx
                    .send(DbusApiEvent::DeviceHotplug((usb_vid, usb_pid), false))
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
            } else {
                warn!(
                    "The device '{}' disappeared during processing of the hotplug event",
                    hotplug_info
                        .devpath
                        .clone()
                        .unwrap_or_default()
                        .to_string_lossy()
                );
            }
        }

        crate::DEVICES
            .write()
            .unwrap()
            .extend(connected_devices.drain(RangeFull));
    }

    Ok(())
}

pub fn resume_from_suspend() -> Result<()> {
    if crate::QUIT.load(Ordering::SeqCst) {
        info!("Ignoring resume from suspend or hibernation event since Eruption is shutting down");
    } else {
        // enumerate devices
        info!("Enumerating connected devices...");

        // initialize devices
        for (_handle, device) in crate::DEVICES.read().unwrap().iter() {
            let make = hwdevices::get_device_make(
                device.read().unwrap().get_usb_vid(),
                device.read().unwrap().get_usb_pid(),
            )
            .unwrap_or("<unknown>");
            let model = hwdevices::get_device_model(
                device.read().unwrap().get_usb_vid(),
                device.read().unwrap().get_usb_pid(),
            )
            .unwrap_or("<unknown>");

            info!("Reinitializing device '{make} {model}'");

            // send initialization handshake
            device
                .write()
                .unwrap()
                .send_init_sequence()
                .unwrap_or_else(|e| error!("Could not initialize the device: {}", e));
        }

        info!("Device enumeration completed");
    }

    Ok(())
}

///
pub struct SdkSupportPlugin {}

impl SdkSupportPlugin {
    pub fn new() -> Self {
        SdkSupportPlugin {}
    }

    #[cfg(not(target_os = "windows"))]
    pub fn initialize_socket() -> Result<()> {
        // unlink any leftover control sockets
        let _result = unlink(constants::CONTROL_SOCKET_NAME)
            .map_err(|e| debug!("Unlink of control socket failed: {}", e));

        // create, bind and store the control socket
        let listener = Socket::new(Domain::UNIX, Type::SEQPACKET, None)?;
        let address = SockAddr::unix(constants::CONTROL_SOCKET_NAME)?;
        listener.bind(&address)?;

        // set permissions of the control socket, allow only root
        let mut perms = fs::metadata(constants::CONTROL_SOCKET_NAME)?.permissions();
        // perms.set_mode(0o660); // don't allow others, only user and group rw
        perms.set_mode(0o666);
        fs::set_permissions(constants::CONTROL_SOCKET_NAME, perms)?;

        LISTENER.write().unwrap().replace(listener);

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn initialize_named_pipe() -> Result<()> {
        // unlink any leftover control sockets
        // let _result = unlink(constants::CONTROL_SOCKET_NAME)
        //     .map_err(|e| debug!("Unlink of control socket failed: {}", e));

        // create, bind and store the control named-pipe

        let listener = PipeListener::bind(Path::new(constants::CONTROL_PIPE_NAME))?;

        // set permissions of the control pipe
        // let mut perms = fs::metadata(constants::CONTROL_SOCKET_NAME)?.permissions();
        // // perms.set_mode(0o660); // don't allow others, only user and group rw
        // perms.set_mode(0o666);
        // fs::set_permissions(constants::CONTROL_SOCKET_NAME, perms)?;

        LISTENER.write().unwrap().replace(listener);

        Ok(())
    }

    pub fn start_control_thread() -> Result<()> {
        let builder = thread::Builder::new().name("control".into());
        builder
            // .stack_size(4096 * 8)
            .spawn(move || loop {
                if crate::QUIT.load(Ordering::SeqCst) {
                    break;
                }

                Self::run_io_loop().unwrap_or_else(|e| {
                    error!("Eruption SDK Plugin thread error: {}", e);
                });
            })
            .unwrap_or_else(|e| {
                error!("Could not spawn a thread: {}", e);
                panic!()
            });

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn run_io_loop() -> Result<()> {
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn run_io_loop() -> Result<()> {
        unsafe fn assume_init(buf: &[MaybeUninit<u8>]) -> &[u8] {
            &*(buf as *const [MaybeUninit<u8>] as *const [u8])
        }

        'IO_LOOP: loop {
            if crate::QUIT.load(Ordering::SeqCst) {
                break 'IO_LOOP;
            }

            if let Some(listener) = LISTENER.read().unwrap().as_ref() {
                listener.listen(1)?;

                match listener.accept() {
                    Ok((socket, sockaddr)) => {
                        let _peer_addr = match sockaddr.as_pathname() {
                            Some(path) => path.to_string_lossy().to_string(),
                            None => String::from("unknown"),
                        };

                        debug!("Eruption SDK client connected");

                        // socket.set_nodelay(true)?; // not supported on AF_UNIX on Linux
                        socket.set_send_buffer_size(constants::NET_BUFFER_CAPACITY)?;
                        socket.set_recv_buffer_size(constants::NET_BUFFER_CAPACITY)?;

                        thread::Builder::new()
                            .name("client".to_string())
                            .spawn(move || -> Result<()> {
                                // connection successful, enter event loop now
                                'EVENT_LOOP: loop {
                                    if crate::QUIT.load(Ordering::SeqCst) {
                                        break 'EVENT_LOOP;
                                    }

                                    // wait for socket to be ready
                                    let mut poll_fds = [PollFd::new(
                                        &socket,
                                        PollFlags::POLLIN
                                            | PollFlags::POLLOUT
                                            | PollFlags::POLLHUP
                                            | PollFlags::POLLERR,
                                    )];

                                    let result = poll(&mut poll_fds, constants::SLEEP_TIME_TIMEOUT as i32)?;

                                    if poll_fds[0].revents().unwrap().contains(PollFlags::POLLHUP)
                                        | poll_fds[0].revents().unwrap().contains(PollFlags::POLLERR)
                                    {
                                        debug!("Eruption SDK client disconnected");

                                        break 'EVENT_LOOP;
                                    }

                                    if result > 0
                                        && poll_fds[0].revents().unwrap().contains(PollFlags::POLLIN)
                                    {
                                        // read data
                                        let mut tmp =
                                            [MaybeUninit::zeroed(); constants::NET_BUFFER_CAPACITY];
                                        match socket.recv(&mut tmp) {
                                            Err(e) => {
                                                error!("Socket receive failed: {}", e);

                                                break 'EVENT_LOOP;
                                            }

                                            Ok(0) => {
                                                debug!("Eruption SDK client disconnected");

                                                break 'EVENT_LOOP;
                                            }

                                            Ok(n) => {
                                                trace!("Read {} bytes from control socket", n);

                                                let tmp = unsafe { assume_init(&tmp[..tmp.len()]) };

                                                if tmp.len() != constants::NET_BUFFER_CAPACITY {
                                                    error!("Buffer length differs from BUFFER_CAPACITY! Length: {}", tmp.len());
                                                }

                                                let result = protocol::Request::decode_length_delimited(
                                                    &mut Cursor::new(&tmp),
                                                );
                                                let request = match result {
                                                    Ok(request) => request,
                                                    Err(e) => {
                                                        error!("Protocol error: {}", e);
                                                        return Err(SdkPluginError::PluginError {
                                                            description:
                                                                "Lost connection to Eruption SDK client"
                                                                    .to_owned(),
                                                        }
                                                        .into());
                                                    }
                                                };

                                                match request.request_message {
                                                    Some(protocol::request::RequestMessage::Noop(
                                                        _message,
                                                    )) => {
                                                        /* Do nothing */

                                                        trace!("NOOP");
                                                    }

                                                    Some(protocol::request::RequestMessage::Status(
                                                        _message,
                                                    )) => {
                                                        trace!("Get Status");

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::Status(
                                                                    protocol::StatusResponse {
                                                                        description: "Eruption".to_string(),
                                                                    },
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(
                                                        protocol::request::RequestMessage::ActiveProfile(
                                                            _message,
                                                        ),
                                                    ) => {
                                                        trace!("Get Active Profile");

                                                        let profile_file = {
                                                            let active_profile =
                                                                &*crate::ACTIVE_PROFILE.read().unwrap();
                                                            match active_profile {
                                                                Some(active_profile) => active_profile
                                                                    .profile_file
                                                                    .to_string_lossy()
                                                                    .to_string(),
                                                                None => "Unknown".to_string(),
                                                            }
                                                        };

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::ActiveProfile(
                                                                    protocol::ActiveProfileResponse {
                                                                        profile_file
                                                                    },
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(
                                                        protocol::request::RequestMessage::SwitchProfile(
                                                            message,
                                                        ),
                                                    ) => {
                                                        trace!("Switch Profile");

                                                        let profile_file =
                                                            PathBuf::from(message.profile_file);
                                                        let switched = crate::switch_profile_please(Some(
                                                            &profile_file,
                                                        ))?;

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::SwitchProfile(
                                                                    protocol::SwitchProfileResponse {
                                                                        switched: switched == SwitchProfileResult::Switched
                                                                    },
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(
                                                        protocol::request::RequestMessage::SetParameters(
                                                            message,
                                                        ),
                                                    ) => {
                                                        let parameter_values: Vec<
                                                            parameters::UntypedParameter,
                                                        > = message
                                                            .parameter_values
                                                            .iter()
                                                            .map(|map| parameters::UntypedParameter {
                                                                name: map.0.to_string(),
                                                                value: map.1.to_string(),
                                                            })
                                                            .collect();
                                                        parameters_util::apply_parameters(
                                                            &message.profile_file,
                                                            &message.script_file,
                                                            &parameter_values,
                                                        )?;

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::SetParameters(
                                                                    protocol::SetParametersResponse {},
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(protocol::request::RequestMessage::SetCanvas(
                                                        message,
                                                    )) => {
                                                        trace!("Set canvas");

                                                        let payload_map = message.canvas;

                                                        if payload_map.len() != constants::CANVAS_SIZE {
                                                            ratelimited::warn!(
                                                                "Length of payload: {} not matching canvas size {}",
                                                                payload_map.len(),
                                                                constants::CANVAS_SIZE
                                                            );
                                                        }

                                                        if !payload_map.is_empty() {
                                                            let mut local_map = LED_MAP.write().unwrap();

                                                            local_map.copy_from_slice(
                                                                &payload_map.chunks(4)
                                                                    .map(|map| RGBA {
                                                                        r: map[0],
                                                                        g: map[1],
                                                                        b: map[2],
                                                                        a: map[3],
                                                                    })
                                                                    .collect::<Vec<_>>(),
                                                            );

                                                            script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
                                                            FRAME_GENERATION_COUNTER_ERUPTION_SDK
                                                                .fetch_add(1, Ordering::SeqCst);
                                                        }

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::SetCanvas(
                                                                    protocol::SetCanvasResponse {},
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(protocol::request::RequestMessage::GetCanvas(
                                                        _message,
                                                    )) => {
                                                        trace!("Get canvas");

                                                        let canvas: Vec<u8> = (*LAST_RENDERED_LED_MAP.read().unwrap()).iter().flat_map(|val| unsafe { any_as_u8_slice(val).to_owned() }).collect();

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::GetCanvas(
                                                                    protocol::GetCanvasResponse { canvas },
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(
                                                        protocol::request::RequestMessage::NotifyHotplug(
                                                            message,
                                                        ),
                                                    ) => {
                                                        trace!("Notify hotplug");

                                                        let payload_hotplug_info = message.payload;

                                                        let config = bincode::config::standard();
                                                        let hotplug_info: HotplugInfo =
                                                            bincode::decode_from_slice(
                                                                &payload_hotplug_info,
                                                                config,
                                                            )?
                                                            .0;

                                                        info!("Hotplug event received, trying to claim the plugged device now...");

                                                        // remove disconnected or failed devices
                                                        // crate::remove_failed_devices()?;

                                                        claim_hotplugged_device(&hotplug_info).unwrap_or_else(|e| {
                                                            error!("Could not initialize the plugged device: {e}");
                                                        });

                                                        // this is required for hotplug to work correctly in case we didn't transfer
                                                        // data to the device for an extended period of time
                                                        script::FRAME_GENERATION_COUNTER
                                                            .fetch_add(1, Ordering::SeqCst);

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::NotifyHotplug(
                                                                    protocol::NotifyHotplugResponse {},
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    Some(
                                                        protocol::request::RequestMessage::NotifyResume(
                                                            _message,
                                                        ),
                                                    ) => {
                                                        trace!("Notify resume from suspend or hibernation");

                                                        info!("Resume event received, trying to reinitialize all devices now...");

                                                        resume_from_suspend().unwrap_or_else(|e| {
                                                            error!("Could not resume at least one device from suspend: {e}");
                                                        });

                                                        // this is required for hotplug to work correctly in case we didn't transfer
                                                        // data to the device for an extended period of time
                                                        script::FRAME_GENERATION_COUNTER
                                                            .fetch_add(1, Ordering::SeqCst);

                                                        let response = protocol::Response {
                                                            response_message: Some(
                                                                protocol::response::ResponseMessage::NotifyHotplug(
                                                                    protocol::NotifyHotplugResponse {},
                                                                ),
                                                            ),
                                                        };

                                                        let mut buf = Vec::new();
                                                        response.encode_length_delimited(&mut buf)?;

                                                        // send data
                                                        match socket.send(&buf) {
                                                            Ok(_n) => {}

                                                            Err(_e) => {
                                                                return Err(SdkPluginError::PluginError {
                                                                    description: "Lost connection to Eruption SDK client".to_owned(),
                                                                }
                                                                    .into());
                                                            }
                                                        }
                                                    }

                                                    None => {
                                                        // not sure how this can happen
                                                        error!(
                                                            "Protocol error: No message in message payload"
                                                        );
                                                        return Err(SdkPluginError::PluginError {
                                                            description: "No message is message payload"
                                                                .to_owned(),
                                                        }
                                                        .into());
                                                    }
                                                }
                                            }
                                        }
                                    }

                                thread::sleep(Duration::from_millis(5));
                            }

                            Ok(())
                        })?;
                    }

                    Err(_e) => {
                        return Err(SdkPluginError::PluginError {
                            description: "Lost connection to Eruption SDK client".to_owned(),
                        }
                        .into());
                    }
                }
            }
        }

        Ok(())
    }
}

impl Plugin for SdkSupportPlugin {
    fn get_name(&self) -> String {
        "SDK Support".to_string()
    }

    fn get_description(&self) -> String {
        "Support for the Eruption SDK".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        #[cfg(not(target_os = "windows"))]
        Self::initialize_socket()?;

        #[cfg(target_os = "windows")]
        Self::initialize_named_pipe()?;

        Self::start_control_thread()?;

        // events::register_observer(|event: &events::Event| {
        //     match event {
        //         events::Event::KeyDown(_index) => {}

        //         events::Event::KeyUp(_index) => {}

        //         _ => (),
        //     };

        //     Ok(true) // event has been processed
        // });

        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let _globals = lua_ctx.globals();

        // let get_current_slot =
        //     lua_ctx.create_function(move |_, ()| Ok(SdkSupportPlugin::get_current_slot()))?;
        // globals.set("get_current_slot", get_current_slot)?;

        Ok(())
    }

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
