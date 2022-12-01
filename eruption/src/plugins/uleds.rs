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

use lazy_static::lazy_static;
use libc::c_char;
use log::*;
use mlua::prelude::*;
use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;
use nix::unistd;
use parking_lot::RwLock;
use std::any::Any;
use std::ffi::CString;
use std::os::unix::prelude::RawFd;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use crate::hwdevices::RGBA;
use crate::plugins::Plugin;
use crate::scripting::script::FRAME_GENERATION_COUNTER;
use crate::{constants, plugins, util, ULEDS_SUPPORT_ACTIVE};

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, Fail)]
// pub enum UledsPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

lazy_static! {
    /// Global LED map, the "canvas"
    pub static ref LED_MAP: Arc<RwLock<Vec<RGBA>>> = Arc::new(RwLock::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; constants::CANVAS_SIZE]));
}

lazy_static! {
    // pub static ref ULEDS_TX: Arc<RwLock<Option<Sender<Message>>>> = Arc::new(RwLock::new(None));

    /// File descriptors for Linux Userspace LEDs subsystem
    pub static ref ULEDS_FDS: Arc<RwLock<Vec<RawFd>>> = Arc::new(RwLock::new(Vec::new()));
}

/// A plugin that creates an interface to the Linux ULEDs subsystem.
/// It allows Eruption to be controlled via in-Kernel LED-triggers.
pub struct UledsPlugin {}

impl UledsPlugin {
    pub fn new() -> Self {
        UledsPlugin {}
    }

    pub fn spawn_uleds_thread() -> Result<()> {
        // let (uleds_tx, uleds_rx) = unbounded();

        thread::Builder::new()
            .name("uleds".into())
            .spawn(move || -> Result<()> {
                #[cfg(feature = "profiling")]
                coz::thread_init();

                // Self::initialize_thread_locals()?;

                if ULEDS_FDS.read().len() > 0 {
                    ULEDS_SUPPORT_ACTIVE.store(true, Ordering::SeqCst);

                    loop {
                        for fd in ULEDS_FDS.read().iter() {
                            let mut buffer = [0u8; 4];
                            let _result = unistd::read(*fd, &mut buffer)?;

                            let brightness = i32::from_ne_bytes(buffer);

                            debug!("ULEDS: value read: {}", brightness);

                            let mut led_map = LED_MAP.write();
                            for (_i, color) in led_map.iter_mut().enumerate() {
                                *color = RGBA {
                                    r: brightness as u8,
                                    g: brightness as u8,
                                    b: brightness as u8,
                                    a: brightness as u8,
                                }
                            }

                            FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }

                Ok(())
            })?;

        // *ULEDS_TX.write() = Some(uleds_tx);

        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for UledsPlugin {
    fn get_name(&self) -> String {
        "Linux ULEDs".to_string()
    }

    fn get_description(&self) -> String {
        "Linux Userspace LEDs interface".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        let filename = PathBuf::from("/dev/uleds");

        if util::file_exists(&filename) {
            let fd = fcntl::open(&filename, OFlag::O_RDWR, Mode::from_bits(0o660).unwrap())?;
            debug!("Successfully opened the ULEDs device");

            //
            let mut dev = UledsUserDev {
                ..Default::default()
            };

            let name = CString::new("eruption::all")?;
            let name = name.as_bytes_with_nul();
            dev.name[0..name.len()]
                .copy_from_slice(&name.iter().map(|&c| c as i8).collect::<Vec<i8>>());

            dev.max_brightness = 255;

            //
            let bytes = unsafe { any_as_u8_slice(&dev) };
            let _result = nix::unistd::write(fd, bytes)?;

            ULEDS_FDS.write().push(fd);

            debug!("Successfully initialized the ULEDs subsystem");

            Ok(())
        } else {
            info!("The ULEDs subsystem is not available on this kernel");

            Ok(())
        }
    }

    fn register_lua_funcs(&self, _lua_ctx: &Lua) -> mlua::Result<()> {
        // let globals = lua_ctx.globals();

        // let get_current_load_avg_1 =
        //     lua_ctx.create_function(|_, ()| Ok(UledsPlugin::get_current_load_avg_1()))?;
        // globals.set("get_current_load_avg_1", get_current_load_avg_1)?;

        Ok(())
    }

    async fn main_loop_hook(&self, _ticks: u64) {}

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    std::slice::from_raw_parts((p as *const T) as *const u8, std::mem::size_of::<T>())
}

/// Struct used to interface to the Linux kernel
#[derive(Debug)]
#[repr(C)]
pub struct UledsUserDev {
    pub name: [c_char; 64],
    pub max_brightness: i32,
}

impl Default for UledsUserDev {
    fn default() -> Self {
        Self {
            name: [0; 64],
            max_brightness: 0,
        }
    }
}
