/* SPDX-License-Identifier: GPL-3.0-or-later */

//! Reusable lighting lifecycle, independent of daemon installation/input handling.

use super::roccat_vulcan_2_protocol::{encode_direct, DeviceInfo};

pub trait LightingTransport {
    fn select_direct_mode(&self) -> Result<(), String>;
    fn set_direct(&self, enabled: bool) -> Result<(), String>;
    fn write_report(&self, report: &[u8]) -> Result<usize, String>;
}

pub struct LightingSession<T: LightingTransport> {
    transport: T,
    info: DeviceInfo,
    brightness: u8,
    active: bool,
    failed: bool,
}

impl<T: LightingTransport> LightingSession<T> {
    pub fn start(transport: T, info: DeviceInfo) -> Result<Self, String> {
        // Broader firmware/layout support requires additional verification.
        if (info.firmware_major, info.firmware_minor, info.layout) != (1, 9, 1) {
            return Err(
                "Lighting currently verified only for firmware 1.09, German ISO layout".into(),
            );
        }
        let session = Self {
            transport,
            info,
            brightness: 100,
            active: true,
            failed: false,
        };
        // Cleanup is already armed if enabling only partially succeeds.
        session.transport.select_direct_mode()?;
        session.transport.set_direct(true)?;
        Ok(session)
    }

    pub fn set_brightness(&mut self, percent: u8) -> Result<(), String> {
        if percent > 100 {
            return Err("Brightness must be between 0 and 100 percent".into());
        }
        self.brightness = percent;
        Ok(())
    }

    pub fn send_colors(&mut self, colors: &[(usize, [u8; 3])]) -> Result<(), String> {
        if !self.active || self.failed {
            return Err("Lighting session is no longer active".into());
        }
        let colors: Vec<_> = colors
            .iter()
            .map(|&(address, rgb)| {
                (
                    address,
                    rgb.map(|channel| (channel as u16 * self.brightness as u16 / 100) as u8),
                )
            })
            .collect();
        let reports = encode_direct(&colors, self.info.length_byte_order())?;
        for report in reports {
            match self.transport.write_report(&report) {
                Ok(length) if length == report.len() => {}
                result => {
                    // Stop accepting further colors after any incomplete update.
                    self.failed = true;
                    let cleanup = self.stop();
                    return Err(format!(
                        "LED report write failed: {result:?}; cleanup: {cleanup:?}"
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.active {
            self.transport.set_direct(false)?;
            self.active = false;
        }
        Ok(())
    }
}

impl<T: LightingTransport> Drop for LightingSession<T> {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            tracing::error!("Could not leave Vulcan II direct mode: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

    #[derive(Default)]
    struct State {
        switches: Vec<bool>,
        reports: Vec<Vec<u8>>,
        short_write: bool,
        enable_error: bool,
        mode_error: bool,
        mode_selections: usize,
    }
    struct Fake(Rc<RefCell<State>>);
    impl LightingTransport for Fake {
        fn select_direct_mode(&self) -> Result<(), String> {
            let mut state = self.0.borrow_mut();
            assert!(state.switches.is_empty());
            state.mode_selections += 1;
            if state.mode_error {
                Err("mode selection failed".into())
            } else {
                Ok(())
            }
        }
        fn set_direct(&self, enabled: bool) -> Result<(), String> {
            let mut state = self.0.borrow_mut();
            state.switches.push(enabled);
            if enabled && state.enable_error {
                Err("enable failed".into())
            } else {
                Ok(())
            }
        }
        fn write_report(&self, report: &[u8]) -> Result<usize, String> {
            let mut state = self.0.borrow_mut();
            state.reports.push(report.to_vec());
            Ok(if state.short_write { 1 } else { report.len() })
        }
    }
    fn info() -> DeviceInfo {
        DeviceInfo {
            firmware_major: 1,
            firmware_minor: 9,
            layout: 1,
        }
    }

    #[test]
    fn lifecycle_scales_brightness_and_disables_once() {
        let state = Rc::new(RefCell::new(State::default()));
        {
            let mut session = LightingSession::start(Fake(state.clone()), info()).unwrap();
            session.set_brightness(50).unwrap();
            assert!(session.set_brightness(101).is_err());
            session.send_colors(&[(0, [200, 100, 50])]).unwrap();
            assert_eq!(&state.borrow().reports[0][5..8], &[100, 50, 25]);
            session.stop().unwrap();
            assert!(session.send_colors(&[]).is_err());
        }
        assert_eq!(state.borrow().switches, [true, false]);
        assert_eq!(state.borrow().reports.len(), 7);
        assert_eq!(state.borrow().mode_selections, 1);
    }

    #[test]
    fn short_write_triggers_cleanup_and_blocks_updates() {
        let state = Rc::new(RefCell::new(State {
            short_write: true,
            ..State::default()
        }));
        let mut session = LightingSession::start(Fake(state.clone()), info()).unwrap();
        assert!(session.send_colors(&[]).is_err());
        assert!(session.send_colors(&[]).is_err());
        assert_eq!(state.borrow().switches, [true, false]);
        assert_eq!(state.borrow().reports.len(), 1);
    }

    #[test]
    fn failed_enable_attempts_cleanup() {
        let state = Rc::new(RefCell::new(State {
            enable_error: true,
            ..State::default()
        }));
        assert!(LightingSession::start(Fake(state.clone()), info()).is_err());
        assert_eq!(state.borrow().switches, [true, false]);
    }

    #[test]
    fn unverified_firmware_is_rejected_without_device_writes() {
        let state = Rc::new(RefCell::new(State::default()));
        let mut info = info();
        info.firmware_minor = 16;
        assert!(LightingSession::start(Fake(state.clone()), info).is_err());
        assert!(state.borrow().switches.is_empty());
        assert_eq!(state.borrow().mode_selections, 0);
    }

    #[test]
    fn failed_mode_selection_attempts_cleanup_without_enable() {
        let state = Rc::new(RefCell::new(State {
            mode_error: true,
            ..State::default()
        }));
        assert!(LightingSession::start(Fake(state.clone()), info()).is_err());
        assert_eq!(state.borrow().switches, [false]);
        assert!(state.borrow().reports.is_empty());
    }
}
