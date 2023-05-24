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

use std::{
    cell::RefCell,
    sync::Arc,
    time::{self, Duration, Instant},
};

use glib::Cast;
use gtk::traits::{ContainerExt, InfoBarExt, LabelExt, WidgetExt};
use lazy_static::{__Deref, lazy_static};
use parking_lot::RwLock;

use crate::{timers::{self, TimerMode}, constants};

thread_local! {
    /// The notification area to display notifications in
    pub static NOTIFICATION_AREA: RefCell<Option<gtk::InfoBar>> = RefCell::new(None);

}

lazy_static! {
    // The instant after that the notification wil be hidden
    pub static ref VISIBLE_SINCE: Arc<RwLock<Option<time::Instant>>> = Arc::new(RwLock::new(None));
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationType {
    Error,
    Warning,
    Info,
    Question,
    Other,
}

impl From<NotificationType> for gtk::MessageType {
    fn from(nt: NotificationType) -> Self {
        match nt {
            NotificationType::Error => gtk::MessageType::Error,
            NotificationType::Warning => gtk::MessageType::Warning,
            NotificationType::Info => gtk::MessageType::Info,
            NotificationType::Question => gtk::MessageType::Question,
            NotificationType::Other => gtk::MessageType::Other,
        }
    }
}

pub fn info(message: &str) {
    show(message, NotificationType::Info);
}

pub fn warn(message: &str) {
    show(message, NotificationType::Warning);
}

pub fn error(message: &str) {
    show(message, NotificationType::Error);
}

pub fn show(message: &str, notification_type: NotificationType) {
    NOTIFICATION_AREA.with(|na| {
        let na = na.borrow();
        let na = na.deref();

        if let Some(na) = na {
            na.content_area().children()[0]
                .downcast_ref::<gtk::Label>()
                .unwrap()
                .set_text(message);

            na.set_message_type(notification_type.into());
            na.set_visible(true);
        }
    });

    schedule_close();
}

pub fn schedule_close() {
    *VISIBLE_SINCE.write() = Some(Instant::now());
}

pub(crate) fn set_notification_area(area: &gtk::InfoBar) {
    NOTIFICATION_AREA.with(|na| {
        *na.borrow_mut() = Some(area.clone());
    });

    let _ = timers::register_timer(
        timers::NOTIFICATION_TIMER_ID,
        TimerMode::Periodic,
        500,
        move || {
            let mut instant = VISIBLE_SINCE.write();

            if let Some(i) = *instant {
                if i.elapsed() >= Duration::from_millis(constants::NOTIFICATION_TIME_MILLIS) {
                    NOTIFICATION_AREA.with(|na| {
                        let na = na.borrow();
                        let na = na.as_ref().unwrap();

                        na.set_visible(false);

                        *instant = None;
                    });
                }
            }

            Ok(())
        },
    );
}
