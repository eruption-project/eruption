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

use std::{collections::HashSet, hash::Hash, sync::Arc};

use async_trait::async_trait;
use dyn_clonable::*;
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;

#[cfg(feature = "sensor-mutter")]
mod mutter;
#[cfg(feature = "sensor-procmon")]
mod process;
#[cfg(feature = "sensor-wayland")]
mod wayland;

#[cfg(feature = "sensor-x11")]
mod x11;
#[cfg(feature = "sensor-mutter")]
pub use mutter::*;
#[cfg(feature = "sensor-procmon")]
pub use process::*;
#[cfg(feature = "sensor-wayland")]
pub use wayland::*;
#[cfg(feature = "sensor-x11")]
pub use x11::*;

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    pub(crate) static ref SENSORS: Arc<Mutex<Vec<Box<dyn Sensor + Send + Sync + 'static>>>> =
        Arc::new(Mutex::new(vec![]));

    /// GLobal configuration of sensors
    pub(crate) static ref SENSORS_CONFIGURATION: Arc<Mutex<HashSet<SensorConfiguration>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum SensorConfiguration {
    AutodetectFailed,

    EnableProcmon,

    EnableMutter,

    EnableWayland,

    EnableX11,
}

impl SensorConfiguration {
    #[allow(unused)]
    pub fn profile_gnome_desktop() -> HashSet<Self> {
        HashSet::from_iter([
            SensorConfiguration::EnableProcmon,
            SensorConfiguration::EnableMutter,
        ])
    }

    #[allow(unused)]
    pub fn profile_generic_wayland_compositor() -> HashSet<Self> {
        HashSet::from_iter([
            SensorConfiguration::EnableProcmon,
            SensorConfiguration::EnableWayland,
        ])
    }

    #[allow(unused)]
    pub fn profile_generic_x11_desktop() -> HashSet<Self> {
        HashSet::from_iter([
            SensorConfiguration::EnableProcmon,
            SensorConfiguration::EnableX11,
        ])
    }

    #[allow(unused)]
    pub fn profile_all_sensors_enabled() -> HashSet<Self> {
        HashSet::from_iter([
            SensorConfiguration::EnableProcmon,
            SensorConfiguration::EnableMutter,
            SensorConfiguration::EnableWayland,
            SensorConfiguration::EnableX11,
        ])
    }
}

#[clonable]
#[async_trait]
pub trait Sensor: Clone {
    fn initialize(&mut self) -> Result<()>;

    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn is_enabled(&self) -> bool;

    fn get_usage_example(&self) -> String;

    fn is_failed(&self) -> bool;
    fn set_failed(&mut self, failed: bool);

    fn is_pollable(&self) -> bool;
    fn poll(&mut self) -> Result<Box<dyn SensorData>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

#[clonable]
#[async_trait]
pub trait WindowSensor: Clone {
    fn initialize(&mut self) -> Result<()>;

    fn get_id(&self) -> String;
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;

    fn get_usage_example(&self) -> String;

    fn is_failed(&self) -> bool;
    fn set_failed(&mut self, failed: bool);

    fn is_pollable(&self) -> bool;
    fn poll(&mut self) -> Result<Box<dyn WindowSensorData>>;

    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait SensorData: std::fmt::Debug {
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait WindowSensorData: SensorData {
    fn window_name(&self) -> Option<&str>;
    fn window_instance(&self) -> Option<&str>;
    fn window_class(&self) -> Option<&str>;
}

/// Register a sensor
#[allow(dead_code)]
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

    #[cfg(feature = "sensor-procmon")]
    register_sensor(ProcessSensor::new());

    #[cfg(feature = "sensor-mutter")]
    register_sensor(MutterSensor::new());

    #[cfg(feature = "sensor-wayland")]
    register_sensor(WaylandSensor::new());

    #[cfg(feature = "sensor-x11")]
    register_sensor(X11Sensor::new());

    // initialize all registered sensors
    for s in SENSORS.lock().iter_mut() {
        s.initialize()?;
    }

    Ok(())
}

/// Find a sensor by its respective id
#[allow(dead_code)]
pub fn find_sensor_by_id(id: &str) -> Option<Box<dyn Sensor + Send + Sync + 'static>> {
    match SENSORS.lock().iter().find(|&e| e.get_id() == id) {
        Some(s) => Some(dyn_clone::clone_box(s.as_ref())),

        None => None,
    }
}
