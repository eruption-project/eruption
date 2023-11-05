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

use std::cell::RefCell;

use dyn_clonable::{clonable, dyn_clone};

#[cfg(feature = "backend-gnome")]
pub mod gnome;
#[cfg(feature = "backend-wayland")]
pub mod wayland;
#[cfg(feature = "backend-x11")]
pub mod x11;

#[cfg(feature = "backend-x11")]
pub use self::x11::*;
#[cfg(feature = "backend-gnome")]
pub use gnome::*;
#[cfg(feature = "backend-wayland")]
pub use wayland::*;

pub mod utils;

type Result<T> = std::result::Result<T, eyre::Error>;

thread_local! {
    pub(crate) static BACKENDS: RefCell<Vec<Box<dyn Backend + 'static>>> = RefCell::new(vec![]);
}

pub type BackendData = String;

#[clonable]
pub trait Backend: Clone {
    fn initialize(&mut self) -> Result<()>;

    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;

    fn is_failed(&self) -> bool;
    fn set_failed(&mut self, failed: bool);

    fn poll(&mut self) -> Result<BackendData>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Register a Backend
#[allow(dead_code)]
pub fn register_backend<S>(backend: S)
where
    S: Backend + Clone + 'static,
{
    let opts = crate::OPTIONS.read().unwrap().as_ref().unwrap().clone();

    if opts.verbose > 0 {
        println!("{} - {}", backend.get_name(), backend.get_description());
    }

    BACKENDS.with(|backends| backends.borrow_mut().push(Box::from(backend)));
}

/// Register all available Backends
pub fn register_backends() -> Result<()> {
    let opts = crate::OPTIONS.read().unwrap().as_ref().unwrap().clone();

    if opts.verbose > 0 {
        println!("Registering backend plugins:");
    }

    #[cfg(feature = "backend-x11")]
    register_backend(X11Backend::new()?);

    #[cfg(feature = "backend-wayland")]
    register_backend(WaylandBackend::new()?);

    #[cfg(feature = "backend-gnome")]
    register_backend(GnomeBackend::new()?);

    // initialize all registered Backends
    BACKENDS.with(|backends| -> Result<()> {
        for backend in backends.borrow_mut().iter_mut() {
            let _ = backend.initialize().map_err(|e| {
                eprintln!("Backend could not be initialized: {e}");
                e
            });
        }

        if opts.verbose > 0 {
            println!("Done initializing all backends");
        }

        Ok(())
    })?;

    Ok(())
}

/// Find a Backend by its respective id
#[allow(dead_code)]
pub fn find_backend_by_id(id: &str) -> Option<Box<dyn Backend + 'static>> {
    BACKENDS.with(|backends| {
        backends
            .borrow()
            .iter()
            .find(|&e| e.get_id() == id)
            .map(|s| dyn_clone::clone_box(s.as_ref()))
    })
}

pub fn get_best_fitting_backend() -> Result<Box<dyn Backend + 'static>> {
    let result = BACKENDS
        .with(|backends| {
            backends
                .borrow()
                .iter()
                .find(|&e| !e.is_failed())
                .map(|s| dyn_clone::clone_box(s.as_ref()))
        })
        .expect("Could not find any suitable screenshot backends!");

    eprintln!("Found suitable backend: {}", result.get_name());

    Ok(result)
}
