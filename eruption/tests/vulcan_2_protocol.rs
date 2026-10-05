/* SPDX-License-Identifier: GPL-3.0-or-later */

#[path = "../src/hwdevices/roccat_vulcan_2_protocol.rs"]
mod protocol;
use protocol as roccat_vulcan_2_protocol;

#[path = "../src/hwdevices/roccat_vulcan_2_session.rs"]
mod session;

#[path = "../src/hwdevices/roccat_vulcan_2_layout.rs"]
mod layout;

use protocol::{encode_direct, DeviceInfo, LengthByteOrder, REPORT_COUNT, REPORT_LENGTH};

#[test]
fn direct_mode_selection_has_verified_header_and_checksum() {
    let report = protocol::direct_mode_report();
    assert_eq!(&report[..9], &[0x11, 0x79, 1, 0, 0x0b, 6, 0x45, 0, 0]);
    assert!(report[9..375].iter().all(|&byte| byte == 0));
    assert_eq!(u16::from_le_bytes([report[375], report[376]]), 225);
}

#[test]
fn linux_key_mapping_is_unique_and_matches_iso_positions() {
    use evdev_rs::enums::EV_KEY::*;
    let keys: std::collections::HashSet<_> = layout::ISO_EV_KEYS.into_iter().collect();
    assert_eq!(keys.len(), layout::NUM_KEYS);
    assert_eq!(layout::key_index(KEY_ESC), 0);
    assert_eq!(layout::key_index(KEY_KPPLUS), 103);
    assert_eq!(layout::key_index(KEY_KPENTER), 104);
    assert_eq!(layout::key_index(KEY_RESERVED), layout::EMPTY);
    for row in 0..layout::NUM_ROWS {
        for col in 0..layout::NUM_COLS {
            assert_eq!(layout::ISO_ROWS[row][col], layout::ISO_COLS[col][row]);
        }
    }
}

#[test]
fn lua_layout_matches_rust_and_has_valid_neighbors() {
    let lua = mlua::Lua::new();
    lua.load(include_str!(
        "../src/scripts/lib/hwdevices/keyboards/roccat_vulcan_2.lua"
    ))
    .exec()
    .unwrap();
    let rows: mlua::Table = lua.globals().get("rows_topology").unwrap();
    let cols: mlua::Table = lua.globals().get("cols_topology").unwrap();
    for row in 0..layout::NUM_ROWS {
        for col in 0..layout::NUM_COLS {
            assert_eq!(
                rows.get::<_, u8>(row * layout::NUM_COLS + col + 1).unwrap(),
                layout::ISO_ROWS[row][col]
            );
            assert_eq!(
                cols.get::<_, u8>(col * layout::NUM_ROWS + row + 1).unwrap(),
                layout::ISO_COLS[col][row]
            );
        }
    }
    let names: mlua::Table = lua.globals().get("key_to_index").unwrap();
    assert_eq!(
        names.clone().pairs::<String, usize>().count(),
        layout::NUM_KEYS
    );
    assert_eq!(names.get::<_, usize>("NUM_PLUS").unwrap(), 104);
    assert_eq!(names.get::<_, usize>("NUM_RETURN").unwrap(), 105);
    let neighbors: mlua::Table = lua.globals().get("neighbor_topology").unwrap();
    for key in 0..layout::NUM_KEYS {
        for slot in 1..=8 {
            let neighbor = neighbors.get::<_, u8>(key * 8 + slot).unwrap();
            assert!(neighbor == layout::EMPTY || (neighbor as usize) < layout::NUM_KEYS);
            assert_ne!(neighbor as usize, key);
        }
    }
}

#[test]
fn iso_layout_has_unique_addresses_within_payload_capacity() {
    let mut addresses = std::collections::HashSet::new();
    for address in layout::ISO_LED_ADDRESSES {
        assert!(addresses.insert(address));
        assert!(encode_direct(
            &[(address as usize, [1, 2, 3])],
            LengthByteOrder::LittleEndian
        )
        .is_ok());
    }
    assert_eq!(addresses.len(), layout::NUM_KEYS);
}

#[test]
fn iso_matrix_covers_every_logical_key_exactly_once() {
    let mut keys = std::collections::HashSet::new();
    for row in layout::ISO_ROWS {
        for key in row.into_iter().filter(|&key| key != layout::EMPTY) {
            assert!((key as usize) < layout::NUM_KEYS);
            assert!(keys.insert(key));
        }
    }
    assert_eq!(keys.len(), layout::NUM_KEYS);
}

#[test]
fn iso_mapping_matches_visually_confirmed_escape_f1_f2() {
    assert_eq!(
        layout::ISO_LED_ADDRESSES[layout::ISO_ROWS[0][0] as usize],
        2
    );
    assert_eq!(
        layout::ISO_LED_ADDRESSES[layout::ISO_ROWS[0][2] as usize],
        13
    );
    assert_eq!(
        layout::ISO_LED_ADDRESSES[layout::ISO_ROWS[0][3] as usize],
        20
    );
}

#[test]
fn actual_german_device_report_is_parsed() {
    let info = DeviceInfo::parse(&[0x09, 0x09, 0x6d, 0x65, 0x65, 0, 1, 0, 1]).unwrap();
    assert_eq!(info.firmware_major, 1);
    assert_eq!(info.firmware_minor, 9);
    assert_eq!(info.layout, 1);
    assert_eq!(info.length_byte_order(), LengthByteOrder::LittleEndian);
}

#[test]
fn newer_firmware_uses_reference_byte_order_rule() {
    let info = DeviceInfo::parse(&[9, 9, 116, 0, 0, 0, 1, 0, 0]).unwrap();
    assert_eq!(info.length_byte_order(), LengthByteOrder::BigEndian);
}

#[test]
fn malformed_information_reports_are_rejected() {
    for len in 0..9 {
        assert!(DeviceInfo::parse(&vec![9; len]).is_err());
    }
    assert!(DeviceInfo::parse(&[9; 10]).is_err());
    assert!(DeviceInfo::parse(&[8, 9, 109, 0, 0, 0, 1, 0, 0]).is_err());
    assert!(DeviceInfo::parse(&[9, 8, 109, 0, 0, 0, 1, 0, 0]).is_err());
}

#[test]
fn unknown_layout_is_preserved_for_caller_validation() {
    let info = DeviceInfo::parse(&[9, 9, 109, 0, 0, 0, 255, 0, 0]).unwrap();
    assert_eq!(info.layout, 255);
}

#[test]
fn black_frame_has_numbered_reports_and_zero_padding() {
    let reports = encode_direct(&[], LengthByteOrder::LittleEndian).unwrap();
    assert_eq!(reports.len(), REPORT_COUNT);
    for (index, report) in reports.iter().enumerate() {
        assert_eq!(report.len(), REPORT_LENGTH);
        assert_eq!(&report[..3], &[0, 0xa1, index as u8 + 1]);
        if index == 0 {
            assert_eq!(&report[3..5], &[0x8c, 0x01]);
        } else {
            assert_eq!(&report[3..5], &[0, 0]);
        }
        assert!(report[5..].iter().all(|&byte| byte == 0));
    }
}

#[test]
fn length_byte_order_is_explicit() {
    let reports = encode_direct(&[], LengthByteOrder::BigEndian).unwrap();
    assert_eq!(&reports[0][3..5], &[0x01, 0x8c]);
}

#[test]
fn rgb_addresses_cross_packet_boundaries_without_touching_headers() {
    let reports = encode_direct(
        &[
            (0, [1, 2, 3]),
            (19, [4, 5, 6]),
            (20, [7, 8, 9]),
            (129, [10, 11, 12]),
        ],
        LengthByteOrder::LittleEndian,
    )
    .unwrap();
    assert_eq!(&reports[0][5..8], &[1, 2, 3]);
    assert_eq!(&reports[0][62..65], &[4, 5, 6]);
    assert_eq!(&reports[1][5..8], &[7, 8, 9]);
    assert_eq!(&reports[6][32..35], &[10, 11, 12]);
    assert!(reports[6][35..].iter().all(|&byte| byte == 0));
}

#[test]
fn invalid_and_duplicate_addresses_are_rejected() {
    assert!(encode_direct(&[(130, [1, 2, 3])], LengthByteOrder::LittleEndian).is_err());
    assert!(encode_direct(&[(usize::MAX, [1, 2, 3])], LengthByteOrder::LittleEndian).is_err());
    assert!(encode_direct(
        &[(0, [1, 2, 3]), (0, [4, 5, 6])],
        LengthByteOrder::LittleEndian
    )
    .is_err());
}
