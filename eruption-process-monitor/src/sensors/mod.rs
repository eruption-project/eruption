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

use std::sync::Arc;

use dyn_clonable::*;
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;

mod process;
mod x11;

pub use process::*;
pub use x11::*;

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    pub(crate) static ref SENSORS: Arc<Mutex<Vec<Box<dyn Sensor + Send + Sync + 'static>>>> =
        Arc::new(Mutex::new(vec![]));
}

#[clonable]
pub trait Sensor: Clone {
    fn initialize(&mut self) -> Result<()>;

    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;

    fn get_usage_example(&self) -> String;

    fn is_pollable(&self) -> bool;
    fn poll(&mut self) -> Result<Box<dyn SensorData>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait SensorData: std::fmt::Debug {
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Register a sensor
pub fn register_sensor<S>(sensor: S)
where
    S: Sensor + Clone + Send + Sync + 'static,
{
    info!("{} - {}", sensor.get_name(), sensor.get_description());

    SENSORS.lock().push(Box::from(sensor));
}

/// Register all available sensors
pub fn register_sensors() -> Result<()> {
    info!("Registering sensor plugins:");

    register_sensor(ProcessSensor::new());
    register_sensor(X11Sensor::new());

    // initialize all registered sensors
    for s in SENSORS.lock().iter_mut() {
        s.initialize()?;
    }

    Ok(())
}

/// Find a sensor by its respective id
pub fn find_sensor_by_id(id: &str) -> Option<Box<dyn Sensor + Send + Sync + 'static>> {
    match SENSORS.lock().iter().find(|&e| e.get_id() == id) {
        Some(s) => Some(dyn_clone::clone_box(&(*s.as_ref()))),

        None => None,
    }
}
