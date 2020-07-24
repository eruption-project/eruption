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

use serde::{Deserialize, Serialize};
use std::io::Result;

#[derive(Debug, Clone)]
pub struct ProcMon {
    nls: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventType {
    Nothing,
    Fork,
    Exec,
    Exit,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub pid: libc::pid_t,
    pub ppid: libc::pid_t,
    pub tgid: libc::pid_t,
}

fn map_int_to_event_type(i: u32) -> EventType {
    match i {
        0x0000_0000 => EventType::Nothing,
        0x0000_0001 => EventType::Fork,
        0x0000_0002 => EventType::Exec,
        0x8000_0000 => EventType::Exit,
        _ => EventType::Invalid,
    }
}

impl ProcMon {
    pub fn new() -> Result<Self> {
        let nls: i32 = unsafe { procmon_sys::nl_connect() };
        unsafe {
            procmon_sys::set_proc_ev_listen(nls, true);
        }

        Ok(ProcMon { nls })
    }

    pub fn wait_for_event(&self) -> Event {
        let mut event = procmon_sys::Event {
            event_type: 0,
            pid: 0,
            ppid: 0,
            tgid: 0,
        };
        unsafe {
            procmon_sys::handle_proc_ev(self.nls, &mut event);
        };

        Event {
            event_type: map_int_to_event_type(event.event_type),
            pid: event.pid,
            ppid: event.ppid,
            tgid: event.tgid,
        }
    }
}
