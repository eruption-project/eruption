/* SPDX-License-Identifier: GPL-3.0-or-later */

//! Experimental Vulcan II lighting driver. Special HID events are not decoded yet.

use super::{
    roccat_vulcan_2_layout as layout, roccat_vulcan_2_protocol as protocol,
    roccat_vulcan_2_session::{LightingSession, LightingTransport},
    *,
};
use evdev_rs::enums::EV_KEY;
use hidapi::{HidApi, HidDevice};
use parking_lot::{Mutex, RwLock};
use std::{any::Any, collections::HashMap, sync::Arc};

pub fn bind_hiddev(api: &HidApi, vid: u16, pid: u16, serial: &str) -> Result<KeyboardDevice> {
    let select = |interface, page| -> Result<hidapi::DeviceInfo> {
        let matches: Vec<_> = api
            .device_list()
            .filter(|d| {
                d.vendor_id() == vid
                    && d.product_id() == pid
                    && d.serial_number().unwrap_or("") == serial
                    && d.interface_number() == interface
                    && d.usage_page() == page
            })
            .collect();
        eyre::ensure!(
            matches.len() == 1,
            "Ambiguous or missing Vulcan II interface"
        );
        Ok(matches[0].clone())
    };
    Ok(Arc::new(RwLock::new(Box::new(RoccatVulcan2 {
        ctrl_info: select(1, 0xff01)?,
        led_info: select(3, 0xff00)?,
        ctrl: Arc::new(Mutex::new(None)),
        led: Arc::new(Mutex::new(None)),
        session: None,
        info: None,
        failed: false,
        brightness: 100,
    }))))
}

struct Transport {
    ctrl: Arc<Mutex<Option<HidDevice>>>,
    led: Arc<Mutex<Option<HidDevice>>>,
}
impl LightingTransport for Transport {
    fn select_direct_mode(&self) -> std::result::Result<(), String> {
        let ctrl = self.ctrl.lock();
        let ctrl = ctrl.as_ref().ok_or("Control interface is closed")?;
        ctrl.send_feature_report(&protocol::direct_mode_report())
            .map_err(|error| error.to_string())?;
        for _ in 0..100 {
            let mut report = [0x04, 0, 0];
            let length = ctrl
                .get_feature_report(&mut report)
                .map_err(|error| error.to_string())?;
            if length >= 2 && report[0] == 0x04 && report[1] == 1 {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        Err("Timed out waiting for Vulcan II firmware readiness".into())
    }
    fn set_direct(&self, enabled: bool) -> std::result::Result<(), String> {
        self.ctrl
            .lock()
            .as_ref()
            .ok_or("Control interface is closed")?
            .send_feature_report(&[0x0e, 5, u8::from(enabled), 0, 0])
            .map_err(|error| error.to_string())?;
        if enabled {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        Ok(())
    }
    fn write_report(&self, report: &[u8]) -> std::result::Result<usize, String> {
        self.led
            .lock()
            .as_ref()
            .ok_or("LED interface is closed")?
            .write(report)
            .map_err(|error| error.to_string())
    }
}

pub struct RoccatVulcan2 {
    ctrl_info: hidapi::DeviceInfo,
    led_info: hidapi::DeviceInfo,
    ctrl: Arc<Mutex<Option<HidDevice>>>,
    led: Arc<Mutex<Option<HidDevice>>>,
    session: Option<LightingSession<Transport>>,
    info: Option<protocol::DeviceInfo>,
    failed: bool,
    brightness: u8,
}

impl DeviceInfoTrait for RoccatVulcan2 {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        [Capability::Keyboard, Capability::RgbLighting].into()
    }
    fn get_device_info(&self) -> Result<DeviceInfo> {
        let info = self.info.ok_or_else(|| eyre::eyre!("Device is not open"))?;
        Ok(DeviceInfo::new(
            info.firmware_major as i32 * 100 + info.firmware_minor as i32,
        ))
    }
    fn get_firmware_revision(&self) -> String {
        self.info.map_or_else(
            || "<unknown>".into(),
            |info| format!("{}.{:02}", info.firmware_major, info.firmware_minor),
        )
    }
}

impl DeviceTrait for RoccatVulcan2 {
    fn get_usb_path(&self) -> String {
        self.led_info.path().to_string_lossy().into_owned()
    }
    fn get_usb_vid(&self) -> u16 {
        self.ctrl_info.vendor_id()
    }
    fn get_usb_pid(&self) -> u16 {
        self.ctrl_info.product_id()
    }
    fn get_serial(&self) -> Option<&str> {
        self.ctrl_info.serial_number()
    }
    fn get_support_script_file(&self) -> String {
        "keyboards/roccat_vulcan_2".into()
    }
    fn open(&mut self, api: &HidApi) -> Result<()> {
        eyre::ensure!(self.ctrl.lock().is_none(), "Device is already open");
        let ctrl = self.ctrl_info.open_device(api)?;
        let led = self.led_info.open_device(api)?;
        let mut report = [0u8; 9];
        report[0] = 9;
        let length = ctrl.get_feature_report(&mut report)?;
        let info = protocol::DeviceInfo::parse(&report[..length]).map_err(eyre::Report::msg)?;
        eyre::ensure!(
            (info.firmware_major, info.firmware_minor, info.layout) == (1, 9, 1),
            "Vulcan II driver currently verified only for firmware 1.09 German ISO layout"
        );
        *self.ctrl.lock() = Some(ctrl);
        *self.led.lock() = Some(led);
        self.info = Some(info);
        self.failed = false;
        Ok(())
    }
    fn send_init_sequence(&mut self) -> Result<()> {
        eyre::ensure!(self.session.is_none(), "Device is already initialized");
        let info = self.info.ok_or_else(|| eyre::eyre!("Device is not open"))?;
        let mut session = LightingSession::start(
            Transport {
                ctrl: self.ctrl.clone(),
                led: self.led.clone(),
            },
            info,
        )
        .map_err(eyre::Report::msg)?;
        session
            .set_brightness(self.brightness)
            .map_err(eyre::Report::msg)?;
        self.session = Some(session);
        Ok(())
    }
    fn close_all(&mut self) -> Result<()> {
        let result = if let Some(mut session) = self.session.take() {
            session.stop().map_err(eyre::Report::msg)
        } else {
            Ok(())
        };
        *self.ctrl.lock() = None;
        *self.led.lock() = None;
        self.info = None;
        result
    }
    fn is_initialized(&self) -> Result<bool> {
        Ok(self.session.is_some() && !self.failed)
    }
    fn has_failed(&self) -> Result<bool> {
        Ok(self.failed)
    }
    fn fail(&mut self) -> Result<()> {
        self.failed = true;
        self.close_all()
    }
    fn write_data_raw(&self, _buf: &[u8]) -> Result<()> {
        eyre::bail!("Raw writes are not supported by the experimental Vulcan II driver");
    }
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>> {
        eyre::ensure!(size <= 4096, "Raw read too large");
        let mut buf = vec![0; size];
        let ctrl = self.ctrl.lock();
        let length = ctrl
            .as_ref()
            .ok_or_else(|| eyre::eyre!("Device is closed"))?
            .read_timeout(&mut buf, 100)?;
        buf.truncate(length);
        Ok(buf)
    }
    fn device_status(&self) -> Result<DeviceStatus> {
        let mut status = HashMap::new();
        status.insert(
            "connected".into(),
            (self.info.is_some() && !self.failed).to_string(),
        );
        status.insert("special_hid_events".into(), "not yet supported".into());
        Ok(DeviceStatus(status))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn as_device(&self) -> &dyn DeviceTrait {
        self
    }
    fn as_device_mut(&mut self) -> &mut dyn DeviceTrait {
        self
    }
    fn as_mouse_device(&self) -> Option<&dyn MouseDeviceTrait> {
        None
    }
    fn as_mouse_device_mut(&mut self) -> Option<&mut dyn MouseDeviceTrait> {
        None
    }
}

impl KeyboardDeviceTrait for RoccatVulcan2 {
    fn set_status_led(&self, _kind: LedKind, _on: bool) -> Result<()> {
        // Leave hardware lock indicators under firmware/kernel control.
        Ok(())
    }
    fn set_local_brightness(&mut self, brightness: i32) -> Result<()> {
        eyre::ensure!((0..=100).contains(&brightness), "Brightness out of range");
        if let Some(session) = &mut self.session {
            session
                .set_brightness(brightness as u8)
                .map_err(eyre::Report::msg)?;
        }
        self.brightness = brightness as u8;
        Ok(())
    }
    fn get_local_brightness(&self) -> Result<i32> {
        Ok(self.brightness as i32)
    }
    fn send_led_map(&mut self, map: &[RGBA]) -> Result<()> {
        eyre::ensure!(
            !self.failed && map.len() >= layout::NUM_KEYS,
            "Invalid device state or short LED map"
        );
        let colors: Vec<_> = layout::ISO_LED_ADDRESSES
            .iter()
            .zip(map)
            .map(|(&address, color)| (address as usize, [color.r, color.g, color.b]))
            .collect();
        let result = self
            .session
            .as_mut()
            .ok_or_else(|| eyre::eyre!("Device is not initialized"))?
            .send_colors(&colors)
            .map_err(eyre::Report::msg);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn set_led_init_pattern(&mut self) -> Result<()> {
        self.send_led_map(
            &[RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }; layout::NUM_KEYS],
        )
    }
    fn set_led_off_pattern(&mut self) -> Result<()> {
        self.set_led_init_pattern()
    }
    fn get_next_event(&self) -> Result<KeyboardHidEvent> {
        self.get_next_event_timeout(100)
    }
    fn get_next_event_timeout(&self, millis: i32) -> Result<KeyboardHidEvent> {
        let ctrl = self.ctrl.lock();
        let mut report = [0u8; 64];
        ctrl.as_ref()
            .ok_or_else(|| eyre::eyre!("Device is closed"))?
            .read_timeout(&mut report, if millis < 0 { 100 } else { millis })?;
        // Do not guess/decode special reports until verified on real hardware.
        Ok(KeyboardHidEvent::Unknown)
    }
    fn ev_key_to_key_index(&self, key: EV_KEY) -> u8 {
        layout::key_index(key)
    }
    fn hid_event_code_to_key_index(&self, _code: &KeyboardHidEventCode) -> u8 {
        layout::EMPTY
    }
    fn hid_event_code_to_report(&self, _code: &KeyboardHidEventCode) -> u8 {
        0
    }
    fn get_num_keys(&self) -> usize {
        layout::NUM_KEYS
    }
    fn get_num_rows(&self) -> usize {
        layout::NUM_ROWS
    }
    fn get_num_cols(&self) -> usize {
        layout::NUM_COLS
    }
    fn get_row_topology(&self, row: usize) -> &'static [u8] {
        layout::ISO_ROWS.get(row).map_or(&[], |row| row.as_slice())
    }
    fn get_col_topology(&self, col: usize) -> &'static [u8] {
        layout::ISO_COLS.get(col).map_or(&[], |col| col.as_slice())
    }
}

impl Drop for RoccatVulcan2 {
    fn drop(&mut self) {
        if let Err(error) = self.close_all() {
            tracing::error!("Vulcan II cleanup failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn driver_is_registered_as_experimental() {
        let drivers = DRIVERS.lock();
        let driver = drivers
            .iter()
            .find(|driver| driver.get_usb_vid() == 0x1e7d && driver.get_usb_pid() == 0x2f4e)
            .unwrap();
        assert_eq!(
            driver
                .as_any()
                .downcast_ref::<KeyboardDriver>()
                .unwrap()
                .status,
            MaturityLevel::Experimental
        );
    }

    #[test]
    #[ignore = "Requires exactly one physical Vulcan II; read-only device query"]
    fn hardware_open_close_without_lighting_writes() {
        let api = HidApi::new().unwrap();
        let device = bind_hiddev(&api, 0x1e7d, 0x2f4e, "").unwrap();
        let mut device = device.write();
        device.open(&api).unwrap();
        assert_eq!(device.get_firmware_revision(), "1.09");
        assert_eq!(device.get_num_keys(), 108);
        assert!(!device.is_initialized().unwrap());
        assert!(!device.has_failed().unwrap());
        assert_eq!(device.get_row_topology(0)[0], 0);
        device.close_all().unwrap();
        assert!(!device.is_initialized().unwrap());
        assert!(device.get_device_info().is_err());
    }

    #[test]
    #[ignore = "Writes LEDs on a physical Vulcan II for three seconds; do not run with another RGB controller"]
    fn hardware_lighting_lifecycle() {
        let api = HidApi::new().unwrap();
        let device = bind_hiddev(&api, 0x1e7d, 0x2f4e, "").unwrap();
        let mut device = device.write();
        device.open(&api).unwrap();
        device.set_local_brightness(100).unwrap();
        device.send_init_sequence().unwrap();
        assert!(device.is_initialized().unwrap());
        device.set_led_init_pattern().unwrap();
        let mut map = [RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }; layout::NUM_KEYS];
        map[layout::key_index(EV_KEY::KEY_ESC) as usize].r = 32;
        map[layout::key_index(EV_KEY::KEY_F1) as usize].g = 32;
        map[layout::key_index(EV_KEY::KEY_F2) as usize].b = 32;
        device.send_led_map(&map).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        device.set_led_off_pattern().unwrap();
        device.close_all().unwrap();
        assert!(!device.is_initialized().unwrap());
        assert!(!device.has_failed().unwrap());
    }
}
