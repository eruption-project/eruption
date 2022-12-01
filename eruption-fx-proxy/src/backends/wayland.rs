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

use image::ImageBuffer;

use super::{Backend, BackendData};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub struct WaylandBackend {
    pub failed: bool,
}

impl WaylandBackend {
    pub fn new() -> Result<Self> {
        Ok(Self { failed: true })
    }
}

impl Backend for WaylandBackend {
    fn initialize(&mut self) -> Result<()> {
        // if we made it up to here, the initialization succeeded
        self.failed = false;

        Ok(())
    }

    fn get_id(&self) -> String {
        "wayland".to_string()
    }

    fn get_name(&self) -> String {
        "Wayland".to_string()
    }

    fn get_description(&self) -> String {
        "Capture the screen's content from a Wayland compositor".to_string()
    }

    fn is_failed(&self) -> bool {
        self.failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    fn poll(&mut self) -> Result<BackendData> {
        wayshot::screenshot()?;

        // TODO: Implement this
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

mod wayshot {
    /*
        Copyright (c) 2022, Aakash Sen Sharma & Contributors
        All rights reserved.
    */

    use nix::{errno::Errno, fcntl, sys::memfd};
    use std::{ffi::CStr, os::unix::prelude::RawFd};

    use std::{
        cell::RefCell,
        fs::File,
        os::unix::prelude::FromRawFd,
        process::exit,
        rc::Rc,
        sync::atomic::{AtomicBool, Ordering},
    };

    use image::{codecs::png::PngEncoder, ColorType::Rgba8, ImageEncoder};
    use memmap2::MmapMut;

    use smithay_client_toolkit::{
        output::OutputInfo,
        reexports::{
            client::{
                protocol::{wl_output::WlOutput, wl_shm, wl_shm::Format},
                Display, GlobalManager, Main,
            },
            protocols::wlr::unstable::screencopy::v1::client::{
                zwlr_screencopy_frame_v1, zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
                zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
            },
        },
    };

    mod output {
        use smithay_client_toolkit::{
            environment,
            environment::Environment,
            output::{with_output_info, OutputHandler, OutputInfo, XdgOutputHandler},
            reexports::{
                client::{protocol::wl_output::WlOutput, Display},
                protocols::unstable::xdg_output::v1::client::zxdg_output_manager_v1::ZxdgOutputManagerV1,
            },
        };

        struct App {
            outputs: OutputHandler,
            xdg_output: XdgOutputHandler,
        }

        environment! {App,
            singles = [
                ZxdgOutputManagerV1 => xdg_output,
            ],
            multis = [
                WlOutput => outputs,
            ]
        }

        pub fn get_valid_outputs(display: Display) -> Vec<(WlOutput, OutputInfo)> {
            let mut queue = display.create_event_queue();
            let attached_display = display.attach(queue.token());

            let (outputs, xdg_output) = XdgOutputHandler::new_output_handlers();
            let mut valid_outputs: Vec<(WlOutput, OutputInfo)> = Vec::new();

            let env = Environment::new(
                &attached_display,
                &mut queue,
                App {
                    outputs,
                    xdg_output,
                },
            )
            .unwrap();

            queue.sync_roundtrip(&mut (), |_, _, _| {}).unwrap();

            for output in env.get_all_outputs() {
                with_output_info(&output, |info| {
                    if !info.obsolete {
                        valid_outputs.push((output.clone(), info.clone()));
                    } else {
                        output.release();
                    }
                });
            }
            valid_outputs
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct FrameFormat {
        format: Format,
        width: u32,
        height: u32,
        stride: u32,
    }

    #[derive(Debug, Copy, Clone)]
    enum FrameState {
        Failed,
        Finished,
    }

    pub fn screenshot() -> super::Result<()> {
        let display = Display::connect_to_env()?;
        let mut event_queue = display.create_event_queue();
        let attached_display = (*display).clone().attach(event_queue.token());

        let globals = GlobalManager::new(&attached_display);
        event_queue.sync_roundtrip(&mut (), |_, _, _| unreachable!())?;

        let valid_outputs = output::get_valid_outputs(display);
        let (output, _): (WlOutput, OutputInfo) = valid_outputs.first().unwrap().clone();

        let frame_formats: Rc<RefCell<Vec<FrameFormat>>> = Rc::new(RefCell::new(Vec::new()));
        let frame_state: Rc<RefCell<Option<FrameState>>> = Rc::new(RefCell::new(None));
        let frame_buffer_done = Rc::new(AtomicBool::new(false));

        let screencopy_manager = globals.instantiate_exact::<ZwlrScreencopyManagerV1>(3)?;

        let cursor_overlay = 0;

        // if args.is_present("output") {
        //     let mut is_present = false;
        //     let valid_outputs = output::get_valid_outputs(display);

        //     for device in valid_outputs {
        //         let (output_device, info) = device;
        //         if info.name == args.value_of("output").unwrap().trim() {
        //             is_present = true;
        //             output = output_device.clone();
        //         }
        //     }
        //     if !is_present {
        //         log::error!(
        //             "\"{}\" is not a valid output.",
        //             args.value_of("output").unwrap().trim()
        //         );
        //         exit(1);
        //     }
        // }

        let frame: Main<ZwlrScreencopyFrameV1> =
            screencopy_manager.capture_output(cursor_overlay, &output);

        frame.quick_assign({
            let frame_formats = frame_formats.clone();
            let frame_state = frame_state.clone();
            let frame_buffer_done = frame_buffer_done.clone();
            move |_frame, event, _| {
                match event {
                    zwlr_screencopy_frame_v1::Event::Buffer {
                        format,
                        width,
                        height,
                        stride,
                    } => {
                        log::debug!("Received buffer event");
                        frame_formats.borrow_mut().push(FrameFormat {
                            format,
                            width,
                            height,
                            stride,
                        });
                    }
                    zwlr_screencopy_frame_v1::Event::Flags { .. } => {
                        log::debug!("Received flags event");
                    }
                    zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                        log::debug!("Received ready event");
                        frame_state.borrow_mut().replace(FrameState::Finished);
                    }
                    zwlr_screencopy_frame_v1::Event::Failed => {
                        log::debug!("Received failed event");
                        frame_state.borrow_mut().replace(FrameState::Failed);
                    }
                    zwlr_screencopy_frame_v1::Event::Damage { .. } => {
                        log::debug!("Received Damaga event");
                    }
                    zwlr_screencopy_frame_v1::Event::LinuxDmabuf { .. } => {
                        log::debug!("Received LinuxDmaBuf event");
                    }
                    zwlr_screencopy_frame_v1::Event::BufferDone => {
                        log::debug!("Received bufferdone event");
                        frame_buffer_done.store(true, Ordering::SeqCst);
                    }
                    _ => unreachable!(),
                };
            }
        });

        while !frame_buffer_done.load(Ordering::SeqCst) {
            event_queue.dispatch(&mut (), |_, _, _| unreachable!())?;
        }

        log::debug!(
            "Received compositor frame buffer formats: {:#?}",
            frame_formats
        );

        let frame_format = frame_formats
            .borrow()
            .iter()
            .filter(|f| {
                matches!(
                    f.format,
                    wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888 | wl_shm::Format::Xbgr8888
                )
            })
            .next()
            .copied();

        log::debug!("Selected frame buffer format: {:#?}", frame_format);

        let frame_format = match frame_format {
            Some(format) => format,
            None => {
                log::error!("No suitable frame format found");
                exit(1);
            }
        };

        let frame_bytes = frame_format.stride * frame_format.height;

        let mem_fd = create_shm_fd()?;
        let mem_file = unsafe { File::from_raw_fd(mem_fd) };

        mem_file.set_len(frame_bytes as u64)?;

        let shm = globals.instantiate_exact::<wl_shm::WlShm>(1)?;
        let pool = shm.create_pool(mem_fd, frame_bytes as i32);
        let buffer = pool.create_buffer(
            0,
            frame_format.width as i32,
            frame_format.height as i32,
            frame_format.stride as i32,
            frame_format.format,
        );

        frame.copy(&buffer);

        loop {
            event_queue.dispatch(&mut (), |_, _, _| {})?;

            if let Some(state) = frame_state.borrow_mut().take() {
                match state {
                    FrameState::Failed => {
                        log::error!("Frame copy failed");
                        break;
                    }

                    FrameState::Finished => {
                        let mut mmap = unsafe { MmapMut::map_mut(&mem_file)? };
                        let data = &mut *mmap;
                        let color_type = match frame_format.format {
                            wl_shm::Format::Argb8888 | wl_shm::Format::Xrgb8888 => {
                                for chunk in data.chunks_exact_mut(4) {
                                    // swap in place (b with r)
                                    chunk.swap(0, 2);
                                }
                                Rgba8
                            }

                            wl_shm::Format::Xbgr8888 => Rgba8,
                            other => {
                                log::error!("Unsupported buffer format: {:?}", other);
                                break;
                            }
                        };

                        let path = "eruption-fx-proxy-screenshot.png";

                        PngEncoder::new(File::create(path)?).write_image(
                            &mmap,
                            frame_format.width,
                            frame_format.height,
                            color_type,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a memfd, only works on Linux
    pub fn create_shm_fd() -> std::io::Result<RawFd> {
        loop {
            match memfd::memfd_create(
                CStr::from_bytes_with_nul(b"eruption-fx-proxy\0").unwrap(),
                memfd::MemFdCreateFlag::MFD_CLOEXEC | memfd::MemFdCreateFlag::MFD_ALLOW_SEALING,
            ) {
                Ok(fd) => {
                    // this is only an optimization, so ignore errors
                    let _ = fcntl::fcntl(
                        fd,
                        fcntl::F_ADD_SEALS(
                            fcntl::SealFlag::F_SEAL_SHRINK | fcntl::SealFlag::F_SEAL_SEAL,
                        ),
                    );

                    return Ok(fd);
                }

                Err(Errno::EINTR) => continue,

                // Err(Errno::ENOSYS) => break,
                Err(errno) => return Err(std::io::Error::from(errno)),
            }
        }
    }
}
