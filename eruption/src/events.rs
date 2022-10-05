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

use crate::{
    constants, dbus_interface, events, macros, script, switch_profile, DbusApiEvent,
    FileSystemEvent, KeyboardDevice, KeyboardHidEvent, MouseDevice, MouseHidEvent, ACTIVE_SLOT,
    DEVICE_STATUS, FAILED_TXS, KEY_STATES, LUA_TXS, MOUSE_MOTION_BUF,
    MOUSE_MOVE_EVENT_LAST_DISPATCHED, REQUEST_FAILSAFE_MODE, REQUEST_PROFILE_RELOAD,
    UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT, UPCALL_COMPLETED_ON_KEY_DOWN,
    UPCALL_COMPLETED_ON_KEY_UP, UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN,
    UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP, UPCALL_COMPLETED_ON_MOUSE_EVENT,
    UPCALL_COMPLETED_ON_MOUSE_HID_EVENT, UPCALL_COMPLETED_ON_MOUSE_MOVE,
};
use flume::Sender;
use lazy_static::lazy_static;
use log::{error, info, trace, warn};
use parking_lot::Mutex;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub enum Event {
    DaemonStartup,
    DaemonShutdown,

    FileSystemEvent(crate::FileSystemEvent),

    KeyboardHidEvent(crate::hwdevices::KeyboardHidEvent),
    MouseHidEvent(crate::hwdevices::MouseHidEvent),

    RawKeyboardEvent(evdev_rs::InputEvent),
    RawMouseEvent(evdev_rs::InputEvent),

    KeyDown(u8),
    KeyUp(u8),

    MouseButtonDown(u8),
    MouseButtonUp(u8),
    MouseMove(u8, i32),
    MouseWheelEvent(u8),
}

pub type Callback = dyn Fn(&Event) -> Result<bool> + Sync + Send + 'static;

lazy_static! {
    static ref INTERNAL_EVENT_OBSERVERS: Arc<Mutex<Vec<Box<Callback>>>> =
        Arc::new(Mutex::new(vec![]));
}

pub fn register_observer<C>(callback: C)
where
    C: Fn(&Event) -> Result<bool> + Sync + Send + 'static,
{
    INTERNAL_EVENT_OBSERVERS.lock().push(Box::from(callback));
}

pub fn notify_observers(event: Event) -> Result<()> {
    for callback in INTERNAL_EVENT_OBSERVERS.lock().iter() {
        callback(&event)?;
    }

    Ok(())
}

/// Process file system related events
pub fn process_filesystem_event(
    fsevent: &FileSystemEvent,
    dbus_api_tx: &Sender<DbusApiEvent>,
) -> Result<()> {
    match fsevent {
        FileSystemEvent::ProfileChanged { action: _, path: _ } => {
            events::notify_observers(events::Event::FileSystemEvent(fsevent.clone()))
                .unwrap_or_else(|e| error!("Error during notification of observers: {}", e));

            dbus_api_tx
                .send(DbusApiEvent::ProfilesChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            // TODO: maybe make this more fine grained
            REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
        }

        FileSystemEvent::ScriptChanged => {
            events::notify_observers(events::Event::FileSystemEvent(fsevent.clone()))
                .unwrap_or_else(|e| error!("Error during notification of observers: {}", e));

            // dbus_api_tx
            //     .send(DbusApiEvent::ScriptChanged)
            //     .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            // TODO: maybe make this more fine grained
            REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
        }
    }

    Ok(())
}

/// Process D-Bus events
pub fn process_dbus_event(
    dbus_event: &dbus_interface::Message,
    dbus_api_tx: &Sender<DbusApiEvent>,
) -> Result<()> {
    match dbus_event {
        dbus_interface::Message::SwitchSlot(slot) => {
            info!("Switching to slot #{}", slot + 1);

            ACTIVE_SLOT.store(*slot, Ordering::SeqCst);
        }

        dbus_interface::Message::SwitchProfile(profile_path) => {
            info!("Loading profile: {}", profile_path.display());

            if let Err(e) = switch_profile(Some(profile_path), dbus_api_tx, true) {
                error!("Could not switch profiles: {}", e);
            }
        }
    }

    Ok(())
}

/// Process a timer tick event
pub fn process_timer_event() -> Result<()> {
    let offset = 0;

    for (index, dev) in crate::KEYBOARD_DEVICES.read().iter().enumerate() {
        let device_status = dev.read().device_status()?;

        DEVICE_STATUS
            .lock()
            .insert((index + offset) as u64, device_status);
    }

    let offset = crate::KEYBOARD_DEVICES.read().len();

    for (index, dev) in crate::MOUSE_DEVICES.read().iter().enumerate() {
        let device_status = dev.read().device_status()?;

        DEVICE_STATUS
            .lock()
            .insert((index + offset) as u64, device_status);
    }

    let offset = crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len();

    for (index, dev) in crate::MISC_DEVICES.read().iter().enumerate() {
        let device_status = dev.read().device_status()?;

        DEVICE_STATUS
            .lock()
            .insert((index + offset) as u64, device_status);
    }

    Ok(())
}

/// Process HID events
pub fn process_keyboard_hid_events(keyboard_device: &KeyboardDevice) -> Result<()> {
    // limit the number of messages that will be processed during this iteration
    let mut loop_counter = 0;

    let mut event_processed = false;

    'HID_EVENTS_LOOP: loop {
        match keyboard_device.read().get_next_event_timeout(0) {
            Ok(result) if result != KeyboardHidEvent::Unknown => {
                event_processed = true;

                events::notify_observers(events::Event::KeyboardHidEvent(result)).unwrap_or_else(
                    |e| {
                        error!(
                            "Error during notification of observers [keyboard_hid_event]: {}",
                            e
                        )
                    },
                );

                *UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock() =
                    LUA_TXS.read().len() - FAILED_TXS.read().len();

                for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                    if !FAILED_TXS.read().contains(&idx) {
                        lua_tx
                            .send(script::Message::KeyboardHidEvent(result))
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending HID event to a Lua VM: {}", e)
                            });
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    // this is required to avoid a deadlock when a Lua script fails
                    // and a key event is pending
                    if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                        *UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock() = 0;
                        break;
                    }

                    let mut pending = UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                // translate HID event to keyboard event
                match result {
                    KeyboardHidEvent::KeyDown { code } => {
                        let index = keyboard_device.read().hid_event_code_to_key_index(&code);
                        if index > 0 {
                            {
                                KEY_STATES.write()[index as usize] = true;
                            }

                            *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() =
                                LUA_TXS.read().len() - FAILED_TXS.read().len();

                            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                                if !FAILED_TXS.read().contains(&idx) {
                                    lua_tx.send(script::Message::KeyDown(index))
                                        .unwrap_or_else(|e| {
                                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                                        });
                                } else {
                                    warn!("Not sending a message to a failed tx");
                                }
                            }

                            // wait until all Lua VMs completed the event handler
                            loop {
                                // this is required to avoid a deadlock when a Lua script fails
                                // and a key event is pending
                                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                                    *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() = 0;
                                    break;
                                }

                                let mut pending = UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();

                                UPCALL_COMPLETED_ON_KEY_DOWN.1.wait_for(
                                    &mut pending,
                                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                                );

                                if *pending == 0 {
                                    break;
                                }
                            }

                            // update AFK timer
                            *crate::LAST_INPUT_TIME.lock() = Instant::now();

                            events::notify_observers(events::Event::KeyDown(index)).unwrap_or_else(
                                |e| error!("Error during notification of observers [keyboard_hid_event]: {}", e),
                            );
                        }
                    }

                    KeyboardHidEvent::KeyUp { code } => {
                        let index = keyboard_device.read().hid_event_code_to_key_index(&code);
                        if index > 0 {
                            {
                                KEY_STATES.write()[index as usize] = false;
                            }

                            *UPCALL_COMPLETED_ON_KEY_UP.0.lock() =
                                LUA_TXS.read().len() - FAILED_TXS.read().len();

                            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                                if !FAILED_TXS.read().contains(&idx) {
                                    lua_tx.send(script::Message::KeyUp(index)).unwrap_or_else(
                                        |e| {
                                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                                        },
                                    );
                                } else {
                                    warn!("Not sending a message to a failed tx");
                                }
                            }

                            // wait until all Lua VMs completed the event handler
                            loop {
                                // this is required to avoid a deadlock when a Lua script fails
                                // and a key event is pending
                                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                                    *UPCALL_COMPLETED_ON_KEY_UP.0.lock() = 0;
                                    break;
                                }

                                let mut pending = UPCALL_COMPLETED_ON_KEY_UP.0.lock();

                                UPCALL_COMPLETED_ON_KEY_UP.1.wait_for(
                                    &mut pending,
                                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                                );

                                if *pending == 0 {
                                    break;
                                }
                            }

                            // update AFK timer
                            *crate::LAST_INPUT_TIME.lock() = Instant::now();

                            events::notify_observers(events::Event::KeyUp(index)).unwrap_or_else(
                                |e| error!("Error during notification of observers [keyboard_hid_event]: {}", e),
                            );
                        }
                    }

                    _ => { /* ignore other events */ }
                }
            }

            Ok(_) => { /* Ignore unknown events */ }

            Err(_e) => {
                event_processed = false;
            }
        }

        if !event_processed || loop_counter >= constants::MAX_EVENTS_PER_ITERATION {
            break 'HID_EVENTS_LOOP; // no more events in queue or iteration limit reached
        }

        loop_counter += 1;
    }

    Ok(())
}

/// Process HID events
pub fn process_mouse_hid_events(mouse_device: &MouseDevice) -> Result<()> {
    // limit the number of messages that will be processed during this iteration
    let mut loop_counter = 0;

    let mut event_processed = false;

    'HID_EVENTS_LOOP: loop {
        match mouse_device.read().get_next_event_timeout(0) {
            Ok(result) if result != MouseHidEvent::Unknown => {
                event_processed = true;

                events::notify_observers(events::Event::MouseHidEvent(result)).unwrap_or_else(
                    |e| {
                        error!(
                            "Error during notification of observers [mouse_hid_event]: {}",
                            e
                        )
                    },
                );

                *UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock() =
                    LUA_TXS.read().len() - FAILED_TXS.read().len();

                for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                    if !FAILED_TXS.read().contains(&idx) {
                        lua_tx
                            .send(script::Message::MouseHidEvent(result))
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending HID event to a Lua VM: {}", e)
                            });
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    // this is required to avoid a deadlock when a Lua script fails
                    // and an event is pending
                    if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                        *UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock() = 0;
                        break;
                    }

                    let mut pending = UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                //     _ => { /* ignore other events */ }
                // }
            }

            Ok(_) => { /* Ignore unknown events */ }

            Err(_e) => {
                event_processed = false;
            }
        }

        if !event_processed || loop_counter >= constants::MAX_EVENTS_PER_ITERATION {
            break 'HID_EVENTS_LOOP; // no more events in queue or iteration limit reached
        }

        loop_counter += 1;
    }

    Ok(())
}

/// Process mouse events
pub fn process_mouse_event(
    raw_event: &evdev_rs::InputEvent,
    mouse_device: &MouseDevice,
) -> Result<()> {
    // send pending mouse events to the Lua VMs and to the event dispatcher

    let mut mirror_event = true;

    // notify all observers of raw events
    events::notify_observers(events::Event::RawMouseEvent(raw_event.clone())).ok();

    if let evdev_rs::enums::EventCode::EV_REL(ref code) = raw_event.clone().event_code {
        match code {
            evdev_rs::enums::EV_REL::REL_X
            | evdev_rs::enums::EV_REL::REL_Y
            | evdev_rs::enums::EV_REL::REL_Z => {
                // mouse move event occurred

                mirror_event = false; // don't mirror pointer motion events, since they are
                                      // already mirrored immediately upon reception

                // accumulate relative changes
                let direction = if *code == evdev_rs::enums::EV_REL::REL_X {
                    MOUSE_MOTION_BUF.write().0 += raw_event.value;

                    1
                } else if *code == evdev_rs::enums::EV_REL::REL_Y {
                    MOUSE_MOTION_BUF.write().1 += raw_event.value;

                    2
                } else if *code == evdev_rs::enums::EV_REL::REL_Z {
                    MOUSE_MOTION_BUF.write().2 += raw_event.value;

                    3
                } else {
                    4
                };

                if *MOUSE_MOTION_BUF.read() != (0, 0, 0)
                    && MOUSE_MOVE_EVENT_LAST_DISPATCHED
                        .read()
                        .elapsed()
                        .as_millis()
                        > constants::EVENTS_UPCALL_RATE_LIMIT_MILLIS.into()
                {
                    *MOUSE_MOVE_EVENT_LAST_DISPATCHED.write() = Instant::now();

                    *UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock() =
                        LUA_TXS.read().len() - FAILED_TXS.read().len();

                    for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                        if !FAILED_TXS.read().contains(&idx) {
                            lua_tx
                                .send(script::Message::MouseMove(
                                    MOUSE_MOTION_BUF.read().0,
                                    MOUSE_MOTION_BUF.read().1,
                                    MOUSE_MOTION_BUF.read().2,
                                ))
                                .unwrap_or_else(|e| {
                                    error!(
                                        "Could not send a pending mouse event to a Lua VM: {}",
                                        e
                                    );
                                });

                            // reset relative motion buffer, since it has been submitted
                            *MOUSE_MOTION_BUF.write() = (0, 0, 0);
                        } else {
                            warn!("Not sending a message to a failed tx");
                        }
                    }

                    // wait until all Lua VMs completed the event handler
                    /*loop {
                        if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                            *UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock() = 0;
                            break;
                        }

                        let mut pending =
                            UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock();

                        UPCALL_COMPLETED_ON_MOUSE_MOVE.1.wait_for(
                            &mut pending,
                            Duration::from_millis(
                                constants::TIMEOUT_CONDITION_MILLIS,
                            ),
                        );

                        if *pending == 0 {
                            break;
                        }
                    }*/
                }

                events::notify_observers(events::Event::MouseMove(direction, raw_event.value))
                    .unwrap_or_else(|e| {
                        error!(
                            "Error during notification of observers [mouse_event]: {}",
                            e
                        )
                    });
            }

            evdev_rs::enums::EV_REL::REL_WHEEL
            | evdev_rs::enums::EV_REL::REL_HWHEEL
            | evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
            | evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES => {
                // mouse scroll wheel event occurred

                let direction;
                if *code == evdev_rs::enums::EV_REL::REL_WHEEL
                    || *code == evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
                {
                    if raw_event.value > 0 {
                        direction = 1
                    } else {
                        direction = 2
                    };
                } else if *code == evdev_rs::enums::EV_REL::REL_HWHEEL
                    || *code == evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES
                {
                    if raw_event.value < 0 {
                        direction = 3
                    } else {
                        direction = 4
                    };
                } else {
                    direction = 5;
                }

                *UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock() =
                    LUA_TXS.read().len() - FAILED_TXS.read().len();

                for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                    if !FAILED_TXS.read().contains(&idx) {
                        lua_tx
                            .send(script::Message::MouseWheelEvent(direction))
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending mouse event to a Lua VM: {}", e)
                            });
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                        *UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock() = 0;
                        break;
                    }

                    let mut pending = UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_MOUSE_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                events::notify_observers(events::Event::MouseWheelEvent(direction)).unwrap_or_else(
                    |e| {
                        error!(
                            "Error during notification of observers [mouse_event]: {}",
                            e
                        )
                    },
                );
            }

            _ => (), // ignore other events
        }
    } else if let evdev_rs::enums::EventCode::EV_KEY(code) = raw_event.clone().event_code {
        // mouse button event occurred

        let is_pressed = raw_event.value > 0;
        let index = mouse_device.read().ev_key_to_button_index(code).unwrap();

        if is_pressed {
            *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() =
                LUA_TXS.read().len() - FAILED_TXS.read().len();

            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                if !FAILED_TXS.read().contains(&idx) {
                    lua_tx
                        .send(script::Message::MouseButtonDown(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending mouse event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                    *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() = 0;
                    break;
                }

                let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock();

                UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::MouseButtonDown(index)).unwrap_or_else(|e| {
                error!(
                    "Error during notification of observers [mouse_event]: {}",
                    e
                )
            });
        } else {
            *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() =
                LUA_TXS.read().len() - FAILED_TXS.read().len();

            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                if !FAILED_TXS.read().contains(&idx) {
                    lua_tx
                        .send(script::Message::MouseButtonUp(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending mouse event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                    *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() = 0;
                    break;
                }

                let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock();

                UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::MouseButtonUp(index)).unwrap_or_else(|e| {
                error!(
                    "Error during notification of observers [mouse_event]: {}",
                    e
                )
            });
        }
    }

    if mirror_event {
        // mirror all events, except pointer motion events.
        // Pointer motion events currently can not be overridden,
        // they are mirrored to the virtual mouse directly after they are
        // received by the mouse plugin. This is done to reduce input lag
        macros::UINPUT_TX
            .read()
            .as_ref()
            .unwrap()
            .send(macros::Message::MirrorMouseEvent(raw_event.clone()))
            .unwrap_or_else(|e| {
                error!(
                    "Error during notification of observers [mouse_event]: {}",
                    e
                )
            });
    }

    Ok(())
}

/// Process mouse events from a secondary sub-device on the primary mouse
// pub fn process_mouse_secondary_events(
//     mouse_rx: &Receiver<Option<evdev_rs::InputEvent>>,
// ) -> Result<()> {
//     // send pending mouse events to the Lua VMs and to the event dispatcher
//     match mouse_rx.recv_timeout(Duration::from_millis(0)) {
//         Ok(result) => {
//             match result {
//                 Some(raw_event) => {
//                     // notify all observers of raw events
//                     events::notify_observers(events::Event::RawMouseEvent(raw_event.clone())).ok();

//                     if let evdev_rs::enums::EventCode::EV_KEY(code) = raw_event.clone().event_code {
//                         // mouse button event occurred

//                         let is_pressed = raw_event.value > 0;
//                         let index = util::ev_key_to_button_index(code).unwrap();

//                         if is_pressed {
//                             *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() =
//                                 LUA_TXS.read().len() - FAILED_TXS.read().len();

//                             for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
//                                 if !FAILED_TXS.read().contains(&idx) {
//                                     lua_tx.send(script::Message::MouseButtonDown(index)).unwrap_or_else(
//                                                 |e| {
//                                                     error!("Could not send a pending mouse event to a Lua VM: {}", e)
//                                                 },
//                                             );
//                                 } else {
//                                     warn!("Not sending a message to a failed tx");
//                                 }
//                             }

//                             // wait until all Lua VMs completed the event handler
//                             loop {
//                                 // this is required to avoid a deadlock when a Lua script fails
//                                 // and an event is pending
//                                 if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
//                                     *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() = 0;
//                                     break;
//                                 }

//                                 let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock();

//                                 UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.wait_for(
//                                     &mut pending,
//                                     Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
//                                 );

//                                 if *pending == 0 {
//                                     break;
//                                 }
//                             }

//                             events::notify_observers(events::Event::MouseButtonDown(index))
//                                 .unwrap_or_else(|e| error!("Error during notification of observers: {}", e));
//                         } else {
//                             *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() =
//                                 LUA_TXS.read().len() - FAILED_TXS.read().len();

//                             for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
//                                 if !FAILED_TXS.read().contains(&idx) {
//                                     lua_tx.send(script::Message::MouseButtonUp(index)).unwrap_or_else(
//                                                 |e| {
//                                                     error!("Could not send a pending mouse event to a Lua VM: {}", e)
//                                                 },
//                                             );
//                                 } else {
//                                     warn!("Not sending a message to a failed tx");
//                                 }
//                             }

//                             // wait until all Lua VMs completed the event handler
//                             loop {
//                                 // this is required to avoid a deadlock when a Lua script fails
//                                 // and an event is pending
//                                 if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
//                                     *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() = 0;
//                                     break;
//                                 }

//                                 let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock();

//                                 UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.wait_for(
//                                     &mut pending,
//                                     Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
//                                 );

//                                 if *pending == 0 {
//                                     break;
//                                 }
//                             }

//                             events::notify_observers(events::Event::MouseButtonUp(index))
//                                 .unwrap_or_else(|e| error!("Error during notification of observers: {}", e));
//                         }
//                     }

//                     // mirror all events, except pointer motion events.
//                     // Pointer motion events currently can not be overridden,
//                     // they are mirrored to the virtual mouse directly after they are
//                     // received by the mouse plugin. This is done to reduce input lag
//                     macros::UINPUT_TX
//                         .lock()
//                         .as_ref()
//                         .unwrap()
//                         .send(macros::Message::MirrorMouseEvent(raw_event.clone()))
//                         .unwrap_or_else(|e| error!("Could not send a pending mouse event: {}", e));

//                     event_processed = true;
//                 }
//             }
//         }
//     }

//     Ok(())
// }

/// Process keyboard events
pub fn process_keyboard_event(
    raw_event: &evdev_rs::InputEvent,
    keyboard_device: &KeyboardDevice,
) -> Result<()> {
    // notify all observers of raw events
    events::notify_observers(events::Event::RawKeyboardEvent(raw_event.clone())).ok();

    if let evdev_rs::enums::EventCode::EV_KEY(ref code) = raw_event.event_code {
        let is_pressed = raw_event.value > 0;
        let index = keyboard_device.read().ev_key_to_key_index(*code);

        trace!("Key index: {:#x}", index);

        if is_pressed {
            *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() = LUA_TXS.read().len() - FAILED_TXS.read().len();

            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                if !FAILED_TXS.read().contains(&idx) {
                    lua_tx
                        .send(script::Message::KeyDown(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                // this is required to avoid a deadlock when a Lua script fails
                // and a key event is pending
                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                    *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() = 0;
                    break;
                }

                let mut pending = UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();

                UPCALL_COMPLETED_ON_KEY_DOWN.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::KeyDown(index)).unwrap_or_else(|e| {
                error!(
                    "Error during notification of observers [keyboard_event]: {}",
                    e
                )
            });
        } else {
            *UPCALL_COMPLETED_ON_KEY_UP.0.lock() = LUA_TXS.read().len() - FAILED_TXS.read().len();

            for (idx, lua_tx) in LUA_TXS.read().iter().enumerate() {
                if !FAILED_TXS.read().contains(&idx) {
                    lua_tx
                        .send(script::Message::KeyUp(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                // this is required to avoid a deadlock when a Lua script fails
                // and a key event is pending
                if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                    *UPCALL_COMPLETED_ON_KEY_UP.0.lock() = 0;
                    break;
                }

                let mut pending = UPCALL_COMPLETED_ON_KEY_UP.0.lock();

                UPCALL_COMPLETED_ON_KEY_UP.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::KeyUp(index)).unwrap_or_else(|e| {
                error!(
                    "Error during notification of observers [keyboard_event]: {}",
                    e
                )
            });
        }
    }

    // handler for Message::MirrorKey will drop the key if a Lua VM
    // called inject_key(..), so that the key won't be reported twice
    macros::UINPUT_TX
        .read()
        .as_ref()
        .unwrap()
        .send(macros::Message::MirrorKey(raw_event.clone()))
        .unwrap_or_else(|e| error!("Could not send a pending keyboard event: {}", e));

    Ok(())
}
