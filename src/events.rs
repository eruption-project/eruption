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

use failure::Error;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Event {
    DaemonStartup,
    DaemonShutdown,

    FileSystemEvent(crate::FileSystemEvent),

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
