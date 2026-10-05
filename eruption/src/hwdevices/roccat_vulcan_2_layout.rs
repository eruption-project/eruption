/* SPDX-License-Identifier: GPL-3.0-or-later */

//! ISO LED layout adapted from OpenRGB RoccatVulcanKeyboardLayouts.h.
//! Original layout by Mola19 (2022), licensed GPL-2.0-or-later.
//! Selected GPLv3-or-later for this adaptation. Reference version:
//! https://github.com/CalcProgrammer1/OpenRGB/blob/master/Controllers/RoccatController/RoccatVulcanKeyboardController/RoccatVulcanKeyboardLayouts.h
//! Logical indices are shared by the Eruption canvas and Lua device topology.
//! Escape/F1/F2 colors and six rows were visually verified on German firmware
//! 1.09; this does not establish every individual key's physical LED position.

pub const NUM_KEYS: usize = 108;
pub const NUM_ROWS: usize = 6;
pub const NUM_COLS: usize = 24;
pub const EMPTY: u8 = 255;

/// Linux key codes follow physical positions, regardless of keycap language.
pub const ISO_EV_KEYS: [evdev_rs::enums::EV_KEY; NUM_KEYS] = {
    use evdev_rs::enums::EV_KEY::*;
    [
        KEY_ESC,
        KEY_GRAVE,
        KEY_TAB,
        KEY_CAPSLOCK,
        KEY_LEFTSHIFT,
        KEY_LEFTCTRL,
        KEY_1,
        KEY_102ND,
        KEY_LEFTMETA,
        KEY_F1,
        KEY_2,
        KEY_Q,
        KEY_A,
        KEY_Z,
        KEY_LEFTALT,
        KEY_F2,
        KEY_3,
        KEY_W,
        KEY_S,
        KEY_X,
        KEY_F3,
        KEY_4,
        KEY_E,
        KEY_D,
        KEY_C,
        KEY_F4,
        KEY_5,
        KEY_R,
        KEY_F,
        KEY_V,
        KEY_6,
        KEY_T,
        KEY_G,
        KEY_B,
        KEY_SPACE,
        KEY_F5,
        KEY_7,
        KEY_Y,
        KEY_H,
        KEY_N,
        KEY_F6,
        KEY_8,
        KEY_U,
        KEY_J,
        KEY_M,
        KEY_F7,
        KEY_9,
        KEY_I,
        KEY_K,
        KEY_COMMA,
        KEY_F8,
        KEY_0,
        KEY_O,
        KEY_L,
        KEY_DOT,
        KEY_RIGHTALT,
        KEY_F9,
        KEY_MINUS,
        KEY_P,
        KEY_SEMICOLON,
        KEY_SLASH,
        KEY_FN,
        KEY_F10,
        KEY_EQUAL,
        KEY_LEFTBRACE,
        KEY_APOSTROPHE,
        KEY_COMPOSE,
        KEY_F11,
        KEY_BACKSPACE,
        KEY_RIGHTBRACE,
        KEY_BACKSLASH,
        KEY_RIGHTSHIFT,
        KEY_F12,
        KEY_ENTER,
        KEY_RIGHTCTRL,
        KEY_SYSRQ,
        KEY_INSERT,
        KEY_DELETE,
        KEY_LEFT,
        KEY_SCROLLLOCK,
        KEY_HOME,
        KEY_END,
        KEY_UP,
        KEY_DOWN,
        KEY_PAUSE,
        KEY_PAGEUP,
        KEY_PAGEDOWN,
        KEY_RIGHT,
        KEY_NUMLOCK,
        KEY_KP7,
        KEY_KP4,
        KEY_KP1,
        KEY_KP0,
        KEY_KPSLASH,
        KEY_KP8,
        KEY_KP5,
        KEY_KP2,
        KEY_KPASTERISK,
        KEY_KP9,
        KEY_KP6,
        KEY_KP3,
        KEY_KPDOT,
        KEY_KPMINUS,
        KEY_KPPLUS,
        KEY_KPENTER,
        KEY_PREVIOUSSONG,
        KEY_PLAYPAUSE,
        KEY_NEXTSONG,
    ]
};

pub fn key_index(key: evdev_rs::enums::EV_KEY) -> u8 {
    ISO_EV_KEYS
        .iter()
        .position(|&candidate| candidate == key)
        .map_or(EMPTY, |index| index as u8)
}

const fn columns() -> [[u8; NUM_ROWS]; NUM_COLS] {
    let mut columns = [[EMPTY; NUM_ROWS]; NUM_COLS];
    let mut row = 0;
    while row < NUM_ROWS {
        let mut column = 0;
        while column < NUM_COLS {
            columns[column][row] = ISO_ROWS[row][column];
            column += 1;
        }
        row += 1;
    }
    columns
}

pub const ISO_COLS: [[u8; NUM_ROWS]; NUM_COLS] = columns();

pub const ISO_LED_ADDRESSES: [u8; NUM_KEYS] = [
    0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0x08, 0x06, 0x07, 0x0D, 0x0E, 0x09, 0x0A, 0x0B, 0x0C, 0x14,
    0x15, 0x0F, 0x10, 0x11, 0x19, 0x1A, 0x16, 0x17, 0x18, 0x1E, 0x1F, 0x1B, 0x1C, 0x1D, 0x24, 0x20,
    0x21, 0x22, 0x23, 0x28, 0x29, 0x25, 0x26, 0x27, 0x2F, 0x30, 0x2A, 0x2B, 0x2C, 0x35, 0x36, 0x31,
    0x32, 0x33, 0x3B, 0x3C, 0x37, 0x38, 0x39, 0x3A, 0x41, 0x42, 0x3D, 0x3E, 0x3F, 0x40, 0x47, 0x48,
    0x43, 0x44, 0x46, 0x4D, 0x50, 0x49, 0x4A, 0x4B, 0x4F, 0x52, 0x4C, 0x53, 0x54, 0x55, 0x56, 0x57,
    0x58, 0x59, 0x5A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x61, 0x62, 0x63, 0x64, 0x65, 0x67, 0x68, 0x69,
    0x6A, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x72, 0x73, 0x75, 0x76, 0x78, 0x79,
];

pub const ISO_ROWS: [[u8; NUM_COLS]; NUM_ROWS] = [
    [
        0, EMPTY, 9, 15, 20, 25, EMPTY, 35, 40, 45, 50, 56, 62, 67, 72, EMPTY, 75, 79, 84, EMPTY,
        105, 106, 107, EMPTY,
    ],
    [
        1, 6, 10, 16, 21, 26, 30, 36, 41, 46, 51, 57, 63, 68, EMPTY, EMPTY, 76, 80, 85, EMPTY, 88,
        93, 97, 102,
    ],
    [
        2, EMPTY, 11, 17, 22, 27, 31, 37, 42, 47, 52, 58, 64, 69, EMPTY, EMPTY, 77, 81, 86, EMPTY,
        89, 94, 98, 103,
    ],
    [
        3, EMPTY, 12, 18, 23, 28, 32, 38, 43, 48, 53, 59, 65, 70, 73, EMPTY, EMPTY, EMPTY, EMPTY,
        EMPTY, 90, 95, 99, EMPTY,
    ],
    [
        4, 7, 13, 19, 24, 29, 33, 39, 44, 49, 54, 60, EMPTY, 71, EMPTY, EMPTY, EMPTY, 82, EMPTY,
        EMPTY, 91, 96, 100, 104,
    ],
    [
        5, 8, 14, EMPTY, EMPTY, EMPTY, 34, EMPTY, EMPTY, EMPTY, 55, 61, 66, EMPTY, 74, EMPTY, 78,
        83, 87, EMPTY, 92, EMPTY, 101, EMPTY,
    ],
];
