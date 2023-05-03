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

#![allow(dead_code)]

use std::cell::RefCell;

thread_local! {
    /// The notification area to display notifications in
    pub static NOTIFICATION_AREA: RefCell<Option<gtk::Statusbar>> = RefCell::new(None);
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationType {
    Error,
    Warning,
    Info,
}

pub fn show_notification(_message: &str, _notification_type: NotificationType) {
    // let notification = match notification_type {
    //     NotificationType::Error => todo!(),
    //     NotificationType::Warning => todo!(),
    //     NotificationType::Info => todo!(),
    // }

    // notification.set_body(message);
    // notification.show();
}

pub fn set_notification_area(area: &gtk::Statusbar) {
    NOTIFICATION_AREA.with(|na| {
        *na.borrow_mut() = Some(area.clone());
    });
}
