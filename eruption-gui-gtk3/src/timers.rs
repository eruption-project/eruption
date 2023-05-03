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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use lazy_static::lazy_static;
use std::cell::RefCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

type Result<T> = std::result::Result<T, eyre::Error>;

// Global timers
pub const HOTPLUG_TIMER_ID: usize = 100;
pub const SETTINGS_TIMER_ID: usize = 150;
pub const COLOR_MAP_TIMER_ID: usize = 200;
// pub const CANVAS_TIMER_ID: usize = 300;
pub const CANVAS_RENDER_TIMER_ID: usize = 400;
pub const CANVAS_ZONES_TIMER_ID: usize = 450;
pub const KEYBOARD_TIMER_ID: usize = 500;
// pub const KEYBOARD_SLOW_TIMER_ID: usize = 600;
pub const KEYBOARD_RENDER_TIMER_ID: usize = 700;
pub const MOUSE_TIMER_ID: usize = 800;
pub const MOUSE_FAST_TIMER_ID: usize = 900;
pub const MOUSE_SLOW_TIMER_ID: usize = 1000;
pub const MOUSE_RENDER_TIMER_ID: usize = 1100;
pub const MISC_TIMER_ID: usize = 1200;
// pub const MISC_SLOW_TIMER_ID: usize = 1300;
pub const MISC_RENDER_TIMER_ID: usize = 1400;
// pub const GLOBAL_CONFIG_TIMER_ID: usize = 1500;

type Callback = dyn Fn() -> Result<()> + 'static;

#[derive(Debug, Clone)]
pub enum TimerMode {
    Periodic,
    ActiveStackPage(usize),
}

thread_local! {
    /// Global timers (ID, mode, interval millis, last fired, callback Fn())
    pub static TIMERS: RefCell<Vec<(usize, TimerMode, u64, Instant, Box<Callback>)>> = RefCell::new(Vec::new());

    /// Global timers (ID bookkeeping)
    pub static REGISTERED_TIMERS: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

lazy_static! {
    /// Global "clear all timers in next iteration" flag
    pub static ref CLEAR_TIMERS: AtomicBool = AtomicBool::new(false);
}

/// Register a timer callback
#[allow(dead_code)]
pub fn clear_timers() -> Result<()> {
    CLEAR_TIMERS.store(true, Ordering::SeqCst);

    Ok(())
}

/// Register a timer callback
pub fn register_timer<T>(id: usize, mode: TimerMode, timeout: u64, callback: T) -> Result<()>
where
    T: Fn() -> Result<()> + 'static,
{
    let mut already_registered = false;

    REGISTERED_TIMERS.with(|f| {
        let mut registered_timers = f.borrow_mut();

        if registered_timers.iter().any(|e| *e == id) {
            tracing::info!("Timer with id {id} has already been registered");

            already_registered = true;
        } else {
            registered_timers.push(id);
        }
    });

    if !already_registered {
        TIMERS.with(|f| {
            if let Ok(mut timers) = f.try_borrow_mut() {
                timers.push((id, mode, timeout, Instant::now(), Box::new(callback)));
            } else {
                tracing::error!("Could not register a timer, the data structure is locked");
            }
        });
    }

    Ok(())
}

/// Remove a previously registered timer
#[allow(dead_code)]
pub fn remove_timer(id: usize) -> Result<()> {
    let mut timer_registered = false;

    REGISTERED_TIMERS.with(|f| {
        let mut registered_timers = f.borrow_mut();

        if registered_timers.iter().any(|e| *e == id) {
            timer_registered = true;
        } else {
            registered_timers.retain(|e| *e != id);
        }
    });

    if timer_registered {
        TIMERS.with(|f| {
            let mut timers = f.borrow_mut();

            timers.retain(|e| e.0 != id);
        });
    }

    Ok(())
}

/// Handle timer callbacks
pub fn handle_timers() -> Result<()> {
    if CLEAR_TIMERS.load(Ordering::SeqCst) {
        CLEAR_TIMERS.store(false, Ordering::SeqCst);

        TIMERS.with(|f| {
            let mut timers = f.borrow_mut();

            timers.clear();
        });

        REGISTERED_TIMERS.with(|f| {
            let mut registered_timers = f.borrow_mut();

            registered_timers.clear();
        });
    }

    TIMERS.with(|f| -> Result<()> {
        let mut timers = f.borrow_mut();

        for (ref _id, ref mode, ref timeout_millis, ref mut last_fired, callback) in
            timers.iter_mut()
        {
            if Instant::now() - *last_fired > Duration::from_millis(*timeout_millis) {
                match mode {
                    TimerMode::Periodic => {
                        let _result = callback().map_err(|e| {
                            tracing::error!("Timer callback failed: {}", e);
                            e
                        });
                    }

                    TimerMode::ActiveStackPage(index) => {
                        if crate::ACTIVE_PAGE.load(Ordering::SeqCst) == *index {
                            let _result = callback().map_err(|e| {
                                tracing::error!("Timer callback failed: {}", e);
                                e
                            });
                        }
                    }
                }

                *last_fired = Instant::now();
            }
        }

        Ok(())
    })?;

    Ok(())
}
