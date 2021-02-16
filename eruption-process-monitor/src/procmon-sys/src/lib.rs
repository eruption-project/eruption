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

#[repr(C)]
pub struct Event {
    pub event_type: u32,
    pub pid: libc::pid_t,
    pub ppid: libc::pid_t,
    pub tgid: libc::pid_t,
}

#[link(name = "procmon")]
extern "C" {
    pub fn nl_connect() -> i32;
    pub fn set_proc_ev_listen(nl_sock: i32, enable: bool) -> i32;
    pub fn handle_proc_ev(nl_sock: i32, event: *mut Event) -> i32;
}
