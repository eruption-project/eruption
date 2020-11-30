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

use crossbeam::channel::{unbounded, Sender};
use evdev_rs::enums::*;
use evdev_rs::{Device, InputEvent, TimeVal, UInputDevice};
use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{any::Any, thread};

use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub enum Message {
    // keyboard related
    MirrorKey(evdev_rs::InputEvent),
    InjectKey { key: u32, down: bool },

    // mouse related
    MirrorMouseEvent(evdev_rs::InputEvent),
    MirrorMouseEventImmediate(evdev_rs::InputEvent),
    InjectButtonEvent { button: u32, down: bool },
    InjectMouseWheelEvent { direction: u32 },
}

#[derive(Debug, thiserror::Error)]
pub enum MacrosPluginError {
    #[error("Could not open the evdev device")]
    EvdevError {},

    #[error("Could not map an evdev event code to a key or button")]
    MappingError {},
}

lazy_static! {
    pub static ref UINPUT_TX: Arc<Mutex<Option<Sender<Message>>>> = Arc::new(Mutex::new(None));
    pub static ref DROP_CURRENT_KEY: AtomicBool = AtomicBool::new(false);
    pub static ref DROP_CURRENT_MOUSE_INPUT: AtomicBool = AtomicBool::new(false);
}

thread_local! {
    static KEYBOARD_DEVICE: RefCell<Option<UInputDevice>> = RefCell::new(None);
    static MOUSE_DEVICE: RefCell<Option<UInputDevice>> = RefCell::new(None);
}

/// Implements support for macros by registering a virtual keyboard and a
/// virtual mouse with the system that mirrors keystrokes and mouse events
/// from the hardware
pub struct MacrosPlugin {}

impl MacrosPlugin {
    pub fn new() -> Self {
        MacrosPlugin {}
    }

    fn initialize_thread_locals() -> Result<()> {
        Self::initialize_virtual_keyboard()?;
        Self::initialize_virtual_mouse()?;

        Ok(())
    }

    fn initialize_virtual_keyboard() -> Result<()> {
        let dev = Device::new().unwrap();

        // setup virtual keyboard device
        dev.set_name("Eruption Virtual Keyboard");
        dev.set_bustype(3);
        dev.set_vendor_id(0xffff);
        dev.set_product_id(0x0123);
        dev.set_version(0x01);

        // configure allowed events
        dev.enable(&EventType::EV_KEY).unwrap();
        dev.enable(&EventType::EV_MSC).unwrap();

        dev.enable(&EventCode::EV_MSC(EV_MSC::MSC_SCAN)).unwrap();
        dev.enable(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)).unwrap();

        // enable FN-F5 - FN-F8
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FILE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HOMEPAGE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MAIL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CALC)).unwrap();

        // enable media keys
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PREVIOUSSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOPCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PLAYPAUSE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NEXTSONG))
            .unwrap();

        // Enable all supported keys; this is used to mirror the hardware device
        // to the virtual keyboard, so that the hardware device can be disabled.

        // Generated via `sudo evtest`
        // Supported events:
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ESC)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_0)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MINUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EQUAL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACKSPACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_TAB)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Q)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_W)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_E)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_R)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_T)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Y)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_U)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_I)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_O)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_P)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTBRACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTBRACE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ENTER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTCTRL))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_A)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_S)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_D)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_G)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_H)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_J)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_K)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_L)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SEMICOLON))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_APOSTROPHE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_GRAVE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTSHIFT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACKSLASH))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_Z)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_X)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_C)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_V)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_B)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_N)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_M)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COMMA)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DOT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SLASH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTSHIFT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPASTERISK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTALT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SPACE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CAPSLOCK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F10)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NUMLOCK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLLOCK))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP9)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPMINUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPPLUS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KP0)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPDOT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_ZENKAKUHANKAKU))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_102ND)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F11)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F12)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RO)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KATAKANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HIRAGANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HENKAN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KATAKANAHIRAGANA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MUHENKAN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPJPCOMMA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPENTER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTCTRL))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPSLASH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SYSRQ)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTALT))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HOME)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAGEUP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_END)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DOWN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAGEDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_INSERT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_DELETE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_MUTE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_VOLUMEUP))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_POWER)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPEQUAL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PAUSE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPCOMMA)).unwrap();
        //dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HANGUEL)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HANJA)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_YEN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_LEFTMETA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_RIGHTMETA))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COMPOSE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_AGAIN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PROPS)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UNDO)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FRONT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_COPY)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_OPEN)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PASTE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FIND)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CUT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_HELP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_CALC)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SLEEP)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_WWW)).unwrap();
        //dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCREENLOCK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_BACK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FORWARD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EJECTCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_NEXTSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PLAYPAUSE))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_PREVIOUSSONG))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_STOPCD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_REFRESH)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_EDIT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLUP))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_SCROLLDOWN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPLEFTPAREN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_KPRIGHTPAREN))
            .unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F13)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F14)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F15)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F16)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F17)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F18)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F19)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F20)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F21)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F22)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F23)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_F24)).unwrap();

        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_FN)).unwrap();

        dev.enable(&EventCode::EV_KEY(EV_KEY::KEY_UNKNOWN)).unwrap();

        match UInputDevice::create_from_device(&dev) {
            Ok(device) => {
                KEYBOARD_DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                Ok(())
            }

            Err(_e) => Err(MacrosPluginError::EvdevError {}.into()),
        }
    }

    fn initialize_virtual_mouse() -> Result<()> {
        let dev = Device::new().unwrap();

        // setup a virtual mouse device
        dev.set_name("Eruption Virtual Mouse");
        dev.set_bustype(3);
        dev.set_vendor_id(0xffff);
        dev.set_product_id(0x0124);
        dev.set_version(0x01);

        // configure allowed events
        dev.enable(&EventType::EV_KEY).unwrap();
        dev.enable(&EventType::EV_REL).unwrap();
        dev.enable(&EventType::EV_MSC).unwrap();

        dev.enable(&EventCode::EV_MSC(EV_MSC::MSC_SCAN)).unwrap();
        dev.enable(&EventCode::EV_SYN(EV_SYN::SYN_REPORT)).unwrap();

        // Enable all supported buttons; this is used to mirror the hardware device
        // to the virtual mouse, so that the hardware device can be disabled.

        // Supported events:
        dev.enable(&EventCode::EV_REL(EV_REL::REL_X)).unwrap();
        dev.enable(&EventCode::EV_REL(EV_REL::REL_Y)).unwrap();
        dev.enable(&EventCode::EV_REL(EV_REL::REL_Z)).unwrap();

        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_LEFT)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_MIDDLE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_RIGHT)).unwrap();

        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_0)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_1)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_2)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_3)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_4)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_5)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_6)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_7)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_8)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_9)).unwrap();

        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_EXTRA)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_SIDE)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_FORWARD)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_BACK)).unwrap();
        dev.enable(&EventCode::EV_KEY(EV_KEY::BTN_TASK)).unwrap();

        dev.enable(&EventCode::EV_REL(EV_REL::REL_WHEEL)).unwrap();
        dev.enable(&EventCode::EV_REL(EV_REL::REL_HWHEEL)).unwrap();
        dev.enable(&EventCode::EV_REL(EV_REL::REL_WHEEL_HI_RES))
            .unwrap();
        dev.enable(&EventCode::EV_REL(EV_REL::REL_HWHEEL_HI_RES))
            .unwrap();

        match UInputDevice::create_from_device(&dev) {
            Ok(device) => {
                MOUSE_DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                Ok(())
            }

            Err(_e) => Err(MacrosPluginError::EvdevError {}.into()),
        }
    }

    fn button_index_to_ev_key(index: u32) -> Result<EV_KEY> {
        match index {
            0 => Ok(EV_KEY::KEY_RESERVED),

            1 => Ok(EV_KEY::BTN_LEFT),
            2 => Ok(EV_KEY::BTN_MIDDLE),
            3 => Ok(EV_KEY::BTN_RIGHT),

            4 => Ok(EV_KEY::BTN_0),
            5 => Ok(EV_KEY::BTN_1),
            6 => Ok(EV_KEY::BTN_2),
            7 => Ok(EV_KEY::BTN_3),
            8 => Ok(EV_KEY::BTN_4),
            9 => Ok(EV_KEY::BTN_5),
            10 => Ok(EV_KEY::BTN_6),
            11 => Ok(EV_KEY::BTN_7),
            12 => Ok(EV_KEY::BTN_8),
            13 => Ok(EV_KEY::BTN_9),

            14 => Ok(EV_KEY::BTN_EXTRA),
            15 => Ok(EV_KEY::BTN_SIDE),
            16 => Ok(EV_KEY::BTN_FORWARD),
            17 => Ok(EV_KEY::BTN_BACK),
            18 => Ok(EV_KEY::BTN_TASK),

            _ => Err(MacrosPluginError::MappingError {}.into()),
        }
    }

    /// Inject a press or release of key `key` into to output of the virtual keyboard
    fn inject_single_key(key: EV_KEY, value: i32, time: &TimeVal) -> Result<()> {
        //let mut do_initialize = false;

        KEYBOARD_DEVICE.with(|dev| {
            let device = dev.borrow();

            if let Some(device) = device.as_ref() {
                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_KEY,
                    event_code: EventCode::EV_KEY(key.clone()),
                    value,
                };

                device.write_event(&event).unwrap();

                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_SYN,
                    event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                    value,
                };

                device.write_event(&event).unwrap();
            } else {
                error!("Inconsistent thread local storage state detected");
                //do_initialize = true;
            }
        });

        //if do_initialize {
        //Self::initialize_thread_locals().unwrap();
        //}

        Ok(())
    }

    /// Inject a press or release of button `button` into to output of the virtual mouse
    fn inject_single_mouse_event(button: EV_KEY, value: i32, time: &TimeVal) -> Result<()> {
        //let mut do_initialize = false;

        MOUSE_DEVICE.with(|dev| {
            let device = dev.borrow();

            if let Some(device) = device.as_ref() {
                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_KEY,
                    event_code: EventCode::EV_KEY(button.clone()),
                    value,
                };

                device.write_event(&event).unwrap();

                let event = InputEvent {
                    time: time.clone(),
                    event_type: EventType::EV_SYN,
                    event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                    value,
                };

                device.write_event(&event).unwrap();
            } else {
                error!("Inconsistent thread local storage state detected");
                //do_initialize = true;
            }
        });

        //if do_initialize {
        //Self::initialize_thread_locals().unwrap();
        //}

        Ok(())
    }

    /// Inject a pre-existing InputEvent into to output of the virtual keyboard device
    fn inject_key_event(event: evdev_rs::InputEvent) -> Result<()> {
        let mut do_initialize = false;

        KEYBOARD_DEVICE.with(|dev| {
            trace!("Injecting: {:?}", event);

            if let Some(device) = dev.borrow().as_ref() {
                device.write_event(&event).unwrap();
            } else {
                do_initialize = true;
            }
        });

        if do_initialize {
            Self::initialize_thread_locals().unwrap();
        }

        Ok(())
    }

    /// Inject a pre-existing InputEvent into to output of the virtual mouse device
    fn inject_mouse_event(event: evdev_rs::InputEvent) -> Result<()> {
        let mut do_initialize = false;

        MOUSE_DEVICE.with(|dev| {
            trace!("Injecting: {:?}", event);

            if let Some(device) = dev.borrow().as_ref() {
                device.write_event(&event).unwrap();
            } else {
                do_initialize = true;
            }
        });

        if do_initialize {
            Self::initialize_thread_locals().unwrap();
        }

        Ok(())
    }

    /// Inject a pre-existing InputEvent into to output of the virtual mouse device
    /// Will send a SYN_REPORT directly after sending `event`
    fn inject_mouse_event_immediate(event: evdev_rs::InputEvent) -> Result<()> {
        let mut do_initialize = false;

        MOUSE_DEVICE.with(|dev| {
            trace!("Injecting: {:?}", event);

            if let Some(device) = dev.borrow().as_ref() {
                let time = event.time.clone();

                device.write_event(&event).unwrap();

                let event = InputEvent {
                    time,
                    event_type: EventType::EV_SYN,
                    event_code: EventCode::EV_SYN(EV_SYN::SYN_REPORT),
                    value: 0,
                };

                device.write_event(&event).unwrap();
            } else {
                do_initialize = true;
            }
        });

        if do_initialize {
            Self::initialize_thread_locals().unwrap();
        }

        Ok(())
    }

    fn spawn_uinput_thread() -> Result<()> {
        let (uinput_tx, uinput_rx) = unbounded();

        thread::Builder::new()
            .name("uinput".into())
            .spawn(move || {
                Self::initialize_thread_locals().unwrap();

                loop {
                    let message = uinput_rx.recv().unwrap();
                    match message {
                        Message::MirrorKey(raw_event) => {
                            if !DROP_CURRENT_KEY.load(Ordering::SeqCst) {
                                Self::inject_key_event(raw_event).unwrap();
                            } else {
                                debug!("Keyboard event has been dropped as requested");
                            }
                        }

                        Message::MirrorMouseEvent(raw_event) => {
                            if !DROP_CURRENT_MOUSE_INPUT.load(Ordering::SeqCst) {
                                Self::inject_mouse_event(raw_event).unwrap();
                            } else {
                                debug!("Mouse event has been dropped as requested");
                            }
                        }

                        Message::MirrorMouseEventImmediate(raw_event) => {
                            Self::inject_mouse_event_immediate(raw_event).unwrap();
                        }

                        Message::InjectKey { key: ev_key, down } => {
                            let key = evdev_rs::enums::int_to_ev_key(ev_key).unwrap_or_else(|| {
                                error!("Invalid key index");
                                panic!()
                            });

                            let value = if down { 1 } else { 0 };

                            let mut time: libc::timeval = libc::timeval {
                                tv_sec: 0,
                                tv_usec: 0,
                            };

                            unsafe {
                                libc::gettimeofday(&mut time, std::ptr::null_mut());
                            }

                            let time = evdev_rs::TimeVal::from_raw(&time);

                            Self::inject_single_key(key, value, &time).unwrap();
                        }

                        Message::InjectButtonEvent { button, down } => {
                            let key = Self::button_index_to_ev_key(button).unwrap_or_else(|e| {
                                error!("Invalid button index: {}", e);
                                panic!()
                            });

                            let value = if down { 1 } else { 0 };

                            let mut time: libc::timeval = libc::timeval {
                                tv_sec: 0,
                                tv_usec: 0,
                            };

                            unsafe {
                                libc::gettimeofday(&mut time, std::ptr::null_mut());
                            }

                            let time = evdev_rs::TimeVal::from_raw(&time);

                            Self::inject_single_mouse_event(key, value, &time).unwrap();
                        }

                        Message::InjectMouseWheelEvent { direction: _ } => {
                            // REL_RESERVED
                        }
                    }
                }
            })?;

        *UINPUT_TX.lock() = Some(uinput_tx);

        Ok(())
    }
}

#[async_trait::async_trait]
impl Plugin for MacrosPlugin {
    fn get_name(&self) -> String {
        "Macros".to_string()
    }

    fn get_description(&self) -> String {
        "Inject programmable keyboard and mouse events".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Self::spawn_uinput_thread()?;

        Ok(())
    }

    fn register_lua_funcs(&self, _lua_ctx: &Lua) -> mlua::Result<()> {
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
