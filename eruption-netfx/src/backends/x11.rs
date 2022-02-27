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

use super::{Backend, BackendData};
use crate::hwdevices::{self, Keyboard};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Clone)]
pub struct X11Backend {
    pub device: Option<Box<dyn Keyboard + Sync + Send>>,
    pub display: Option<xwrap::Display>,

    pub failed: bool,
}

impl X11Backend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            device: None,
            display: None,
            failed: true,
        })
    }
}

impl Backend for X11Backend {
    fn initialize(&mut self) -> Result<()> {
        self.failed = true;

        let opts = crate::OPTIONS.read().as_ref().unwrap().clone();

        self.device = Some(hwdevices::get_keyboard_device(&opts.model)?);
        self.display = Some(xwrap::Display::open(None).unwrap());

        // if we made it up to here, the initialization succeeded
        self.failed = false;

        Ok(())
    }

    fn get_id(&self) -> String {
        "x11".to_string()
    }

    fn get_name(&self) -> String {
        "X11".to_string()
    }

    fn get_description(&self) -> String {
        "Capture the screen's content from an X11 server".to_string()
    }

    fn is_failed(&self) -> bool {
        self.failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    fn poll(&mut self) -> Result<BackendData> {
        let display = self.display.as_ref().expect("Display is not initialized");
        let device = self.device.as_ref().expect("Device is not initialized");

        let window = display.get_default_root();

        let window_rect = display.get_window_rect(window);

        let sel = xwrap::Rect {
            x: 0,
            y: 0,
            w: window_rect.w,
            h: window_rect.h,
        };

        let image = display
            .get_image(window, sel, xwrap::ALL_PLANES, x11::xlib::ZPixmap)
            .unwrap();

        let commands = super::utils::process_screenshot(&image, &device)?;

        Ok(commands)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

pub mod xwrap {
    // This Source Code Form is subject to the terms of the Mozilla Public
    // License, v. 2.0. If a copy of the MPL was not distributed with this
    // file, You can obtain one at http://mozilla.org/MPL/2.0/.

    // Based on project: https://github.com/neXromancers/shotgun

    use std::ffi;
    use std::mem;
    use std::os::raw;
    use std::ptr;
    use std::slice;

    use image::Rgba;
    use image::RgbaImage;
    use x11::xlib;
    use x11::xrandr;

    #[derive(Copy, Clone, Debug)]
    pub struct Rect {
        pub x: i32,
        pub y: i32,
        pub w: i32,
        pub h: i32,
    }

    pub const ALL_PLANES: libc::c_ulong = !0;

    #[derive(Debug, Clone)]
    pub struct Display {
        handle: *mut xlib::Display,
    }

    pub struct Image {
        handle: *mut xlib::XImage,
    }

    pub struct ScreenRectIter<'a> {
        dpy: &'a Display,
        res: *mut xrandr::XRRScreenResources,
        crtcs: &'a [xrandr::RRCrtc],
        i: usize,
    }

    impl Display {
        pub fn open(name: Option<ffi::CString>) -> Option<Display> {
            unsafe {
                let name = match name {
                    None => ptr::null(),
                    Some(cstr) => cstr.as_ptr(),
                };
                let d = xlib::XOpenDisplay(name);

                if d.is_null() {
                    return None;
                }

                Some(Display { handle: d })
            }
        }

        pub fn get_default_root(&self) -> xlib::Window {
            unsafe { xlib::XDefaultRootWindow(self.handle) }
        }

        pub fn get_window_rect(&self, window: xlib::Window) -> Rect {
            unsafe {
                let mut attrs = mem::MaybeUninit::uninit();
                xlib::XGetWindowAttributes(self.handle, window, attrs.as_mut_ptr());
                let attrs = attrs.assume_init();

                let mut root = 0;
                let mut parent = 0;
                let mut children: *mut xlib::Window = ptr::null_mut();
                let mut nchildren = 0;
                xlib::XQueryTree(
                    self.handle,
                    window,
                    &mut root,
                    &mut parent,
                    &mut children,
                    &mut nchildren,
                );
                if !children.is_null() {
                    xlib::XFree(children as *mut raw::c_void);
                }

                let mut x = attrs.x;
                let mut y = attrs.y;

                if parent != 0 {
                    let mut child = 0;
                    xlib::XTranslateCoordinates(
                        self.handle,
                        parent,
                        root,
                        attrs.x,
                        attrs.y,
                        &mut x,
                        &mut y,
                        &mut child,
                    );
                }

                Rect {
                    x,
                    y,
                    w: attrs.width,
                    h: attrs.height,
                }
            }
        }

        pub fn get_image(
            &self,
            window: xlib::Window,
            rect: Rect,
            plane_mask: libc::c_ulong,
            format: libc::c_int,
        ) -> Option<Image> {
            unsafe {
                let image = xlib::XGetImage(
                    self.handle,
                    window,
                    rect.x,
                    rect.y,
                    rect.w as libc::c_uint,
                    rect.h as libc::c_uint,
                    plane_mask,
                    format,
                );

                if image.is_null() {
                    return None;
                }

                Some(Image::from_raw_ximage(image))
            }
        }

        // pub fn get_screen_rects(&self, root: xlib::Window) -> Option<ScreenRectIter<'_>> {
        //     unsafe {
        //         let xrr_res = xrandr::XRRGetScreenResourcesCurrent(self.handle, root);

        //         if xrr_res.is_null() {
        //             return None;
        //         }

        //         Some(ScreenRectIter {
        //             dpy: &self,
        //             res: xrr_res,
        //             crtcs: slice::from_raw_parts((*xrr_res).crtcs, (*xrr_res).ncrtc as usize),
        //             i: 0,
        //         })
        //     }
        // }
    }

    impl Drop for Display {
        fn drop(&mut self) {
            unsafe {
                xlib::XCloseDisplay(self.handle);
            }
        }
    }

    impl Image {
        pub fn from_raw_ximage(ximage: *mut xlib::XImage) -> Image {
            Image { handle: ximage }
        }

        #[allow(clippy::wrong_self_convention)]
        pub fn into_image_buffer(&self) -> Option<RgbaImage> {
            unsafe {
                // Extract values from the XImage into our own scope
                macro_rules! get {
                ($($a:ident),+) => ($(let $a = (*self.handle).$a;)+);
            }
                get!(
                    width,
                    height,
                    byte_order,
                    depth,
                    bytes_per_line,
                    bits_per_pixel,
                    red_mask,
                    green_mask,
                    blue_mask
                );

                // Pixel size
                let stride = match (depth, bits_per_pixel) {
                    (24, 24) => 3,
                    (24, 32) | (32, 32) => 4,
                    _ => return None,
                };

                // Compute subpixel offsets into each pixel according the the bitmasks X gives us
                // Only 8 bit, byte-aligned values are supported
                // Truncate masks to the lower 32 bits as that is the maximum pixel size
                macro_rules! channel_offset {
                    ($mask:expr) => {
                        match (byte_order, $mask & 0xFFFFFFFF) {
                            (0, 0xFF) | (1, 0xFF000000) => 0,
                            (0, 0xFF00) | (1, 0xFF0000) => 1,
                            (0, 0xFF0000) | (1, 0xFF00) => 2,
                            (0, 0xFF000000) | (1, 0xFF) => 3,
                            _ => return None,
                        }
                    };
                }
                let red_offset = channel_offset!(red_mask);
                let green_offset = channel_offset!(green_mask);
                let blue_offset = channel_offset!(blue_mask);
                let alpha_offset = channel_offset!(!(red_mask | green_mask | blue_mask));

                // Wrap the pixel buffer into a slice
                let size = (bytes_per_line * height) as usize;
                let data = slice::from_raw_parts((*self.handle).data as *const u8, size);

                // Finally, generate the image object
                Some(RgbaImage::from_fn(width as u32, height as u32, |x, y| {
                    macro_rules! subpixel {
                        ($channel_offset:ident) => {
                            data[(y * bytes_per_line as u32 + x * stride as u32 + $channel_offset)
                                as usize]
                        };
                    }
                    Rgba([
                        subpixel!(red_offset),
                        subpixel!(green_offset),
                        subpixel!(blue_offset),
                        // Make the alpha channel fully opaque if none is provided
                        if depth == 24 {
                            0xFF
                        } else {
                            subpixel!(alpha_offset)
                        },
                    ])
                }))
            }
        }
    }

    impl Drop for Image {
        fn drop(&mut self) {
            unsafe {
                xlib::XDestroyImage(self.handle);
            }
        }
    }

    impl<'a> Iterator for ScreenRectIter<'a> {
        type Item = Rect;

        fn next(&mut self) -> Option<Self::Item> {
            if self.i >= self.crtcs.len() {
                return None;
            }

            unsafe {
                // TODO Handle failure here?
                let crtc = xrandr::XRRGetCrtcInfo((*self.dpy).handle, self.res, self.crtcs[self.i]);
                let x = (*crtc).x;
                let y = (*crtc).y;
                let w = (*crtc).width;
                let h = (*crtc).height;
                xrandr::XRRFreeCrtcInfo(crtc);

                self.i += 1;

                //Some((w as i32, h as i32, x as i32, y as i32))
                Some(Rect {
                    x,
                    y,
                    w: w as i32,
                    h: h as i32,
                })
            }
        }
    }

    impl<'a> Drop for ScreenRectIter<'a> {
        fn drop(&mut self) {
            unsafe {
                xrandr::XRRFreeScreenResources(self.res);
            }
        }
    }

    // pub fn parse_geometry(g: ffi::CString) -> Rect {
    //     unsafe {
    //         let mut x = 0;
    //         let mut y = 0;
    //         let mut w = 0;
    //         let mut h = 0;
    //         xlib::XParseGeometry(
    //             g.as_ptr() as *const raw::c_char,
    //             &mut x,
    //             &mut y,
    //             &mut w,
    //             &mut h,
    //         );

    //         Rect {
    //             x: x,
    //             y: y,
    //             w: w as i32,
    //             h: h as i32,
    //         }
    //     }
    // }
}
