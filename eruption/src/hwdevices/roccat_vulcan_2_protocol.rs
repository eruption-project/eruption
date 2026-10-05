/* SPDX-License-Identifier: GPL-3.0-or-later */

//! Initial Vulcan II direct-lighting packet encoder, without hardware access.
//!
//! Protocol reference: OpenRGB's RoccatVulcanKeyboardController::SendColors
//! (GPL-2.0-or-later). This is an independent implementation of the framing.
//! LED indices are wire addresses, not Eruption canvas indices. Firmware byte
//! order must be selected by the caller after device information is verified.

pub const FRAME_LENGTH: u16 = 396;
pub const REPORT_LENGTH: usize = 65;
pub const REPORT_COUNT: usize = 7;
const PAYLOAD_PER_REPORT: usize = 60;
const PAYLOAD_LENGTH: usize = FRAME_LENGTH as usize - 4;

/// Firmware mode selection, based on OpenRGB's SendMode (Mola19).
/// Disabling direct transfer afterwards returns to a default effect;
/// arbitrary previous firmware effects cannot currently be restored.
pub fn direct_mode_report() -> [u8; 377] {
    let mut report = [0u8; 377];
    report[..7].copy_from_slice(&[0x11, 0x79, 0x01, 0, 0x0b, 0x06, 0x45]);
    let checksum: u16 = report[..375].iter().map(|&byte| u16::from(byte)).sum();
    report[375..].copy_from_slice(&checksum.to_le_bytes());
    report
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub firmware_major: u8,
    pub firmware_minor: u8,
    /// Raw layout identifier; do not silently substitute another layout.
    pub layout: u8,
}

impl DeviceInfo {
    /// Parses the nine-byte response to feature report 0x09.
    pub fn parse(report: &[u8]) -> Result<Self, &'static str> {
        if report.len() != 9 || report[0] != 0x09 || report[1] != 9 {
            return Err("Invalid Vulcan II device information report");
        }
        Ok(Self {
            firmware_major: report[2] / 100,
            firmware_minor: report[2] % 100,
            layout: report[6],
        })
    }

    /// Matches OpenRGB's firmware-dependent direct-mode framing rule.
    /// Only firmware 1.09 has been queried on our physical device so far.
    pub fn length_byte_order(self) -> LengthByteOrder {
        if self.firmware_major >= 1 && self.firmware_minor >= 16 {
            LengthByteOrder::BigEndian
        } else {
            LengthByteOrder::LittleEndian
        }
    }
}

/// Encodes RGB values by hardware address. Duplicate addresses are rejected.
/// Each report includes HID API's leading zero for an unnumbered output report.
/// Unspecified addresses and all padding are black/zero.
pub fn encode_direct(
    colors: &[(usize, [u8; 3])],
    byte_order: LengthByteOrder,
) -> Result<[[u8; REPORT_LENGTH]; REPORT_COUNT], &'static str> {
    let mut payload = [0u8; PAYLOAD_LENGTH];
    let mut seen = [false; PAYLOAD_LENGTH / 3];
    for &(index, rgb) in colors {
        if index >= seen.len() {
            return Err("LED address exceeds the direct-lighting payload");
        }
        if seen[index] {
            return Err("Duplicate LED address");
        }
        seen[index] = true;
        payload[index * 3..index * 3 + 3].copy_from_slice(&rgb);
    }

    let mut reports = [[0u8; REPORT_LENGTH]; REPORT_COUNT];
    for (index, report) in reports.iter_mut().enumerate() {
        report[1] = 0xa1;
        report[2] = index as u8 + 1;
    }
    reports[0][3..5].copy_from_slice(&match byte_order {
        LengthByteOrder::LittleEndian => FRAME_LENGTH.to_le_bytes(),
        LengthByteOrder::BigEndian => FRAME_LENGTH.to_be_bytes(),
    });
    for (index, &byte) in payload.iter().enumerate() {
        reports[index / PAYLOAD_PER_REPORT][5 + index % PAYLOAD_PER_REPORT] = byte;
    }
    Ok(reports)
}
