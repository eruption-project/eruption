-- SPDX-License-Identifier: GPL-3.0-or-later
--
-- This file is part of Eruption.
--
-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
--
-- Copyright (c) 2019-2022, The Eruption Development Team
--
-- global config
ENABLE_FUNCTION_KEYS = true
ENABLE_MEDIA_KEYS = true
ENABLE_MACRO_KEYS = true

-- HID key codes
FN_KEY = 119
GAME_MODE_KEY = 96
EASY_SHIFT_KEY = 255

-- character to key index mapping
key_to_index = {}

-- F Keys
key_to_index['F1'] = 12
key_to_index['F2'] = 18
key_to_index['F3'] = 24
key_to_index['F4'] = 29
key_to_index['F5'] = 49
key_to_index['F6'] = 54
key_to_index['F7'] = 60
key_to_index['F8'] = 66
key_to_index['F9'] = 79
key_to_index['F10'] = 85
key_to_index['F11'] = 86
key_to_index['F12'] = 87

-- Special Keys
key_to_index['ESC'] = 1
key_to_index['PRINT'] = 100
key_to_index['ROLL'] = 104
key_to_index['GAME_MODE'] = 104
key_to_index['PAUSE'] = 109
key_to_index['BACKSPACE'] = 88
key_to_index['TAB'] = 3
key_to_index['RETURN'] = 89
key_to_index['CAPS_LOCK'] = 4
key_to_index['LEFT_SHIFT'] = 5
key_to_index['RIGHT_SHIFT'] = 83
key_to_index['LEFT_CTRL'] = 6
key_to_index['MOD_LEFT'] = 11
key_to_index['LEFT_ALT'] = 17
key_to_index['SPACE'] = 38
key_to_index['RIGHT_ALT'] = 71
key_to_index['FN'] = 77
key_to_index['RIGHT_MENU'] = 84
key_to_index['RIGHT_CTRL'] = 90
key_to_index['INSERT'] = 101
key_to_index['POS1'] = 105
key_to_index['PGUP'] = 110
key_to_index['PGDWN'] = 111
key_to_index['DEL'] = 102
key_to_index['END'] = 106

-- Num Pad
key_to_index['NUM'] = 114
key_to_index['NUM_DIV'] = 120
key_to_index['NUM_MULT'] = 125
key_to_index['NUM_MINUS'] = 130
key_to_index['NUM_PLUS'] = 131
key_to_index['NUM_RETURN'] = 132
key_to_index['NUM_COMMA'] = 129
key_to_index['NUM_0'] = 118
key_to_index['NUM_1'] = 117
key_to_index['NUM_2'] = 123
key_to_index['NUM_3'] = 128
key_to_index['NUM_4'] = 116
key_to_index['NUM_5'] = 122
key_to_index['NUM_6'] = 127
key_to_index['NUM_7'] = 115
key_to_index['NUM_8'] = 121
key_to_index['NUM_9'] = 126

-- Arrow Keys
key_to_index['UP'] = 107
key_to_index['DOWN'] = 108
key_to_index['LEFT'] = 103
key_to_index['RIGHT'] = 112

-- Numbers
key_to_index['1'] = 7
key_to_index['2'] = 13
key_to_index['3'] = 19
key_to_index['4'] = 25
key_to_index['5'] = 30
key_to_index['6'] = 34
key_to_index['7'] = 50
key_to_index['8'] = 55
key_to_index['9'] = 61
key_to_index['0'] = 67

-- Letters
key_to_index['ß'] = 73
key_to_index['Q'] = 8
key_to_index['W'] = 14
key_to_index['E'] = 20
key_to_index['R'] = 26
key_to_index['T'] = 31
key_to_index['Z'] = 35
key_to_index['U'] = 51
key_to_index['I'] = 56
key_to_index['O'] = 62
key_to_index['P'] = 68
key_to_index['Ü'] = 74
key_to_index['A'] = 9
key_to_index['S'] = 15
key_to_index['D'] = 21
key_to_index['F'] = 27
key_to_index['G'] = 32
key_to_index['H'] = 36
key_to_index['J'] = 52
key_to_index['K'] = 57
key_to_index['L'] = 63
key_to_index['Ö'] = 69
key_to_index['Ä'] = 75
key_to_index['Y'] = 16
key_to_index['X'] = 22
key_to_index['C'] = 28
key_to_index['V'] = 33
key_to_index['B'] = 37
key_to_index['N'] = 53
key_to_index['M'] = 58

-- Others
key_to_index['^'] = 2
key_to_index['`'] = 80
key_to_index['+'] = 81
key_to_index['#'] = 97
key_to_index['<'] = 10
key_to_index[','] = 64
key_to_index['.'] = 70
key_to_index['-'] = 76

-- support functions
function device_specific_key_highlights() end

-- coordinates to key index mapping
coordinates_to_index = {

    -- ISO model
    0x01, 0x0c, 0x12, 0x18, 0x31, 0x36, 0x3c, 0x42, 0x4f, 0x55, 0x56, 0x67
    -- TODO: ... complete this ...

    -- TODO: ANSI model
}

keys_per_col = {
    0x06, 0x05, 0x06, 0x05, 0x06, 0x05, 0x05, 0x09, 0x06, 0x07, 0x05, 0x06,
    0x05, 0x05, 0x06, 0x04, 0x04, 0x04, 0x04, 0x04, 0x05, 0x05, 0x03
}

num_keys = get_num_keys()

-- rows
num_rows = 6
max_keys_per_row = 22
rows_topology = {

    -- ISO model
    0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55,
    0x56, 0x63, 0x67, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x06,
    0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57,
    0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff, 0x02, 0x07, 0x0d, 0x13,
    0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x58, 0x65, 0x69,
    0x6e, 0x72, 0x78, 0x7d, 0x82, 0xff, 0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f,
    0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x60, 0x73, 0x79, 0x7e, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0x04, 0x09, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34,
    0x39, 0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff,
    0xff, 0xff, 0x05, 0x0a, 0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b,
    0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    -- ANSI model
    0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55,
    0x56, 0x63, 0x67, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x06,
    0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57,
    0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff, 0x02, 0x07, 0x0d, 0x13,
    0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x51, 0x65, 0x69,
    0x6e, 0x72, 0x78, 0x7d, 0x82, 0xff, 0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f,
    0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x58, 0x73, 0x79, 0x7e, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0x04, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34, 0x39,
    0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff,
    0xff, 0xff, 0x05, 0x0a, 0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b,
    0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
}

-- columns
num_cols = 21
max_keys_per_col = 6
cols_topology = {

    -- ISO model
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0xff,
    0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0xff,
    0x17, 0x18, 0x19, 0x1a, 0x1b, 0xff, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0xff,
    0x21, 0x22, 0x23, 0x24, 0x25, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0xff,
    0x36, 0x37, 0x38, 0x39, 0x3b, 0xff, 0x3c, 0x3d, 0x3e, 0x3f, 0x41, 0xff,
    0x42, 0x43, 0x44, 0x45, 0x46, 0xff, 0x48, 0x49, 0x4a, 0x4b, 0x4e, 0x4c,
    0x4f, 0x50, 0x53, 0x54, 0x60, 0xff, 0x52, 0x55, 0x57, 0x58, 0x59, 0xff,
    0x56, 0x63, 0x64, 0x65, 0x66, 0xff, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0xff,
    0x6c, 0x6d, 0x6e, 0x6f, 0xff, 0xff, 0x71, 0x72, 0x73, 0x74, 0x75, 0xff,
    0x77, 0x78, 0x79, 0x7a, 0xff, 0xff, 0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0xff,
    0x81, 0x82, 0x83, 0xff, 0xff, 0xff, -- ANSI model
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0a, 0xff, 0xff,
    0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0xff,
    0x17, 0x18, 0x19, 0x1a, 0x1b, 0xff, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0xff,
    0x21, 0x22, 0x23, 0x24, 0x25, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0xff,
    0x36, 0x37, 0x38, 0x39, 0x3b, 0xff, 0x3c, 0x3d, 0x3e, 0x3f, 0x41, 0xff,
    0x42, 0x43, 0x44, 0x45, 0x46, 0xff, 0x48, 0x49, 0x4a, 0x4b, 0x4e, 0x4c,
    0x4f, 0x50, 0x53, 0x54, 0xff, 0xff, 0x51, 0x52, 0x55, 0x57, 0x58, 0x59,
    0x56, 0x63, 0x64, 0x65, 0x66, 0xff, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0xff,
    0x6c, 0x6d, 0x6e, 0x6f, 0xff, 0xff, 0x71, 0x72, 0x73, 0x74, 0x75, 0xff,
    0x77, 0x78, 0x79, 0x7a, 0xff, 0xff, 0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0xff,
    0x81, 0x82, 0x83, 0xff, 0xff, 0xff
}

-- neighbor tables
max_neigh = 10
neighbor_topology = {

    -- ISO model
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- sentinel
    0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x00
    0x00, 0x02, 0x06, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x01
    0x01, 0x03, 0x06, 0x07, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x02
    0x02, 0x04, 0x08, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x03
    0x03, 0x05, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x04
    0x04, 0x0a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x05
    0x01, 0x02, 0x07, 0x0c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x06
    0x02, 0x06, 0x08, 0x0c, 0x0d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x07
    0x03, 0x07, 0x09, 0x0e, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x08
    0x03, 0x04, 0x08, 0x0a, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x09
    0x05, 0x09, 0x10, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0a
    0x0c, 0x11, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0b
    0x06, 0x07, 0x0b, 0x0d, 0x12, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0c
    0x07, 0x0c, 0x0e, 0x12, 0x13, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0d
    0x08, 0x0d, 0x0f, 0x14, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0e
    0x08, 0x09, 0x0e, 0x10, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0f
    0x0a, 0x0f, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x10
    0x0b, 0x12, 0x17, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x11
    0x0c, 0x0d, 0x11, 0x13, 0x18, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x12
    0x0d, 0x12, 0x14, 0x18, 0x19, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x13
    0x0e, 0x13, 0x15, 0x1a, 0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x14
    0x0e, 0x0f, 0x10, 0x14, 0x1b, 0x25, 0xff, 0xff, 0xff, 0xff, -- 0x15
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x16
    0x11, 0x18, 0x1c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x17
    0x12, 0x13, 0x17, 0x19, 0x1d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x18
    0x13, 0x18, 0x1a, 0x1d, 0x1e, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x19
    0x14, 0x19, 0x1b, 0x1f, 0x20, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1a
    0x14, 0x15, 0x1a, 0x20, 0x25, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1b
    0x17, 0x1d, 0x30, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1c
    0x18, 0x19, 0x1c, 0x1e, 0x21, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1d
    0x19, 0x1d, 0x1f, 0x21, 0x22, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1e
    0x1a, 0x1e, 0x20, 0x23, 0x24, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1f
    0x1a, 0x1b, 0x1f, 0x24, 0x25, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x20
    0x1d, 0x1e, 0x22, 0x30, 0x31, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x21
    0x1e, 0x21, 0x23, 0x31, 0x32, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x22
    0x1f, 0x22, 0x24, 0x33, 0x34, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x23
    0x1f, 0x20, 0x23, 0x25, 0x34, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x24
    0x10, 0x15, 0x1b, 0x20, 0x24, 0x34, 0x39, 0x3f, 0x46, 0xff, -- 0x25
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x26
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x27
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x28
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x29
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2f
    0x1c, 0x21, 0x31, 0x35, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x30
    0x21, 0x22, 0x30, 0x32, 0x35, 0x36, 0xff, 0xff, 0xff, 0xff, -- 0x31
    0x22, 0x31, 0x33, 0x36, 0x37, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x32
    0x23, 0x32, 0x34, 0x38, 0x39, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x33
    0x23, 0x24, 0x25, 0x33, 0x39, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x34
    0x30, 0x31, 0x36, 0x3b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x35
    0x31, 0x32, 0x35, 0x37, 0x3b, 0x3c, 0xff, 0xff, 0xff, 0xff, -- 0x36
    0x32, 0x36, 0x38, 0x3c, 0x3d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x37
    0x33, 0x37, 0x39, 0x3e, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x38
    0x25, 0x33, 0x34, 0x38, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x39
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3a
    0x35, 0x36, 0x3c, 0x41, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3b
    0x36, 0x37, 0x3b, 0x3d, 0x41, 0x42, 0xff, 0xff, 0xff, 0xff, -- 0x3c
    0x37, 0x3c, 0x3e, 0x42, 0x43, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3d
    0x38, 0x3d, 0x3f, 0x44, 0x45, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3e
    0x25, 0x38, 0x39, 0x3e, 0x45, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3f
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x40
    0x3b, 0x3c, 0x42, 0x4e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x41
    0x3c, 0x3d, 0x41, 0x43, 0x48, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x42
    0x3d, 0x42, 0x44, 0x48, 0x49, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x43
    0x3e, 0x43, 0x45, 0x4a, 0x4b, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x44
    0x3e, 0x3f, 0x44, 0x46, 0x4b, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x45
    0xff, 0x45, 0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x46
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x47
    0x42, 0x43, 0x49, 0x4e, 0x4f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x48
    0x43, 0x48, 0x4a, 0x4f, 0x50, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x49
    0x44, 0x49, 0x4b, 0x52, 0x60, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4a
    0x44, 0x45, 0x4a, 0x4c, 0x52, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4b
    0x46, 0x4b, 0x53, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4d
    0x41, 0x48, 0x54, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4e
    0x48, 0x49, 0x50, 0x54, 0x57, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4f
    0x49, 0x4f, 0x57, 0x58, 0x60, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x50
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x51
    0x4a, 0x4b, 0x53, 0x58, 0x59, 0x60, 0xff, 0xff, 0xff, 0xff, -- 0x52
    0x52, 0x59, 0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x53
    0x4e, 0x4f, 0x55, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x54
    0x54, 0x56, 0x57, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x55
    0x55, 0x57, 0x63, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x56
    0x4f, 0x50, 0x55, 0x56, 0x58, 0x64, 0xff, 0xff, 0xff, 0xff, -- 0x57
    0x50, 0x52, 0x57, 0x60, 0x65, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x58
    0x52, 0x53, 0x66, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x59
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5f
    0x4a, 0x50, 0x52, 0x58, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x60
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x61
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x62
    0x56, 0x64, 0x67, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x63
    0x57, 0x63, 0x65, 0x68, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x64
    0x58, 0x64, 0x69, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x65
    0x59, 0x6b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x66
    0x63, 0x68, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x67
    0x64, 0x67, 0x69, 0x6d, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x68
    0x65, 0x68, 0x6e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x69
    0x6b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6a
    0x66, 0x6a, 0x6f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6b
    0x67, 0x6d, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6c
    0x68, 0x6c, 0x6e, 0x71, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6d
    0x69, 0x6d, 0x72, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6e
    0x6b, 0x75, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6f
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x70
    0x6d, 0x72, 0x77, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x71
    0x6e, 0x71, 0x73, 0x78, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x72
    0x72, 0x74, 0x79, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x73
    0x73, 0x75, 0x7a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x74
    0x6f, 0x74, 0x7a, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x75
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x76
    0x71, 0x78, 0x7c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x77
    0x72, 0x77, 0x79, 0x7d, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x78
    0x73, 0x78, 0x7a, 0x7e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x79
    0x74, 0x75, 0x79, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7b
    0x77, 0x7d, 0x81, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7c
    0x78, 0x7c, 0x7e, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7d
    0x79, 0x7d, 0x7f, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7e
    0x7a, 0x7e, 0x80, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7f
    0x75, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x80
    0x7c, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x81
    0x7d, 0x7e, 0x81, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x82
    0x7f, 0x80, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x83
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x84
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x85
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x86
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x87
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x88
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x89
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8f
    -- ANSI model
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- sentinel
    0x01, 0x06, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x00
    0x00, 0x02, 0x06, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x01
    0x01, 0x03, 0x06, 0x07, 0x0c, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x02
    0x02, 0x04, 0x07, 0x08, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x03
    0x03, 0x05, 0x08, 0x0a, 0x0f, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x04
    0x04, 0x0a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x05
    0x00, 0x01, 0x02, 0x07, 0x0b, 0x0c, 0xff, 0xff, 0xff, 0xff, -- 0x06
    0x02, 0x03, 0x06, 0x08, 0x0c, 0x0d, 0xff, 0xff, 0xff, 0xff, -- 0x07
    0x03, 0x04, 0x07, 0x0d, 0x0e, 0x0f, 0xff, 0xff, 0xff, 0xff, -- 0x08
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x09
    0x04, 0x05, 0x0f, 0x10, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0a
    0x06, 0x0c, 0x11, 0x12, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x0b
    0x06, 0x07, 0x0b, 0x0d, 0x11, 0x12, 0xff, 0xff, 0xff, 0xff, -- 0x0c
    0x07, 0x08, 0x0c, 0x0e, 0x12, 0x13, 0xff, 0xff, 0xff, 0xff, -- 0x0d
    0x08, 0x0d, 0x0f, 0x13, 0x14, 0x15, 0xff, 0xff, 0xff, 0xff, -- 0x0e
    0x04, 0x08, 0x0a, 0x0e, 0x10, 0x15, 0xff, 0xff, 0xff, 0xff, -- 0x0f
    0x0a, 0x0f, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x10
    0x0b, 0x0c, 0x12, 0x17, 0x18, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x11
    0x0b, 0x0c, 0x0d, 0x11, 0x13, 0x17, 0x18, 0xff, 0xff, 0xff, -- 0x12
    0x0d, 0x0e, 0x12, 0x14, 0x18, 0x19, 0xff, 0xff, 0xff, 0xff, -- 0x13
    0x0e, 0x13, 0x15, 0x19, 0x1a, 0x1b, 0xff, 0xff, 0xff, 0xff, -- 0x14
    0x0e, 0x0f, 0x10, 0x14, 0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x15
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x16
    0x11, 0x12, 0x18, 0x1c, 0x1d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x17
    0x11, 0x12, 0x13, 0x17, 0x19, 0x1c, 0x1d, 0xff, 0xff, 0xff, -- 0x18
    0x13, 0x14, 0x18, 0x1a, 0x1d, 0x1e, 0xff, 0xff, 0xff, 0xff, -- 0x19
    0x14, 0x19, 0x1b, 0x1e, 0x1f, 0x20, 0xff, 0xff, 0xff, 0xff, -- 0x1a
    0x14, 0x15, 0x1a, 0x20, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1b
    0x17, 0x18, 0x1d, 0x21, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x1c
    0x17, 0x18, 0x19, 0x1c, 0x1e, 0x21, 0xff, 0xff, 0xff, 0xff, -- 0x1d
    0x19, 0x1a, 0x1d, 0x1f, 0x21, 0x22, 0xff, 0xff, 0xff, 0xff, -- 0x1e
    0x1a, 0x1e, 0x20, 0x22, 0x23, 0x24, 0xff, 0xff, 0xff, 0xff, -- 0x1f
    0x1a, 0x1b, 0x1f, 0x24, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x20
    0x1d, 0x1e, 0x22, 0x30, 0x31, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x21
    0x1e, 0x1f, 0x21, 0x23, 0x31, 0x32, 0xff, 0xff, 0xff, 0xff, -- 0x22
    0x1f, 0x22, 0x24, 0x32, 0x33, 0x34, 0xff, 0xff, 0xff, 0xff, -- 0x23
    0x1f, 0x20, 0x23, 0x25, 0x34, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x24
    0x20, 0x24, 0x34, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x25
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x26
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x27
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x28
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x29
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x2f
    0x21, 0x31, 0x35, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x30
    0x21, 0x22, 0x30, 0x32, 0x35, 0x36, 0xff, 0xff, 0xff, 0xff, -- 0x31
    0x22, 0x23, 0x31, 0x33, 0x36, 0x37, 0xff, 0xff, 0xff, 0xff, -- 0x32
    0x23, 0x32, 0x34, 0x37, 0x38, 0x39, 0xff, 0xff, 0xff, 0xff, -- 0x33
    0x23, 0x24, 0x25, 0x33, 0x39, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x34
    0x30, 0x31, 0x36, 0x3b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x35
    0x31, 0x32, 0x35, 0x37, 0x3b, 0x3c, 0xff, 0xff, 0xff, 0xff, -- 0x36
    0x32, 0x33, 0x36, 0x38, 0x3c, 0x3d, 0xff, 0xff, 0xff, 0xff, -- 0x37
    0x33, 0x37, 0x39, 0x3d, 0x3e, 0x3f, 0xff, 0xff, 0xff, 0xff, -- 0x38
    0x33, 0x34, 0x38, 0x3f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x39
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3a
    0x35, 0x36, 0x3c, 0x41, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3b
    0x36, 0x37, 0x3b, 0x3d, 0x41, 0x42, 0xff, 0xff, 0xff, 0xff, -- 0x3c
    0x37, 0x38, 0x3c, 0x3e, 0x42, 0x43, 0xff, 0xff, 0xff, 0xff, -- 0x3d
    0x38, 0x3d, 0x3f, 0x43, 0x44, 0x45, 0xff, 0xff, 0xff, 0xff, -- 0x3e
    0x38, 0x39, 0x3e, 0x45, 0x46, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x3f
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x40
    0x3b, 0x3c, 0x42, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x41
    0x3c, 0x3d, 0x41, 0x43, 0x48, 0x4e, 0xff, 0xff, 0xff, 0xff, -- 0x42
    0x3d, 0x3e, 0x42, 0x44, 0x48, 0x49, 0xff, 0xff, 0xff, 0xff, -- 0x43
    0x3e, 0x43, 0x45, 0x49, 0x4a, 0x4b, 0xff, 0xff, 0xff, 0xff, -- 0x44
    0x3e, 0x3f, 0x44, 0x46, 0x4b, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x45
    0x3f, 0x45, 0x4b, 0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x46
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x47
    0x42, 0x43, 0x49, 0x4e, 0x4f, 0x54, 0xff, 0xff, 0xff, 0xff, -- 0x48
    0x43, 0x44, 0x48, 0x4a, 0x4f, 0x50, 0xff, 0xff, 0xff, 0xff, -- 0x49
    0x44, 0x49, 0x4b, 0x50, 0x52, 0x58, 0xff, 0xff, 0xff, 0xff, -- 0x4a
    0x44, 0x45, 0x4a, 0x4c, 0x52, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4b
    0x45, 0x46, 0x4b, 0x52, 0x53, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4d
    0x42, 0x48, 0x4f, 0x54, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x4e
    0x48, 0x49, 0x4e, 0x50, 0x54, 0x55, 0x57, 0xff, 0xff, 0xff, -- 0x4f
    0x49, 0x4a, 0x4f, 0x51, 0x57, 0x58, 0xff, 0xff, 0xff, 0xff, -- 0x50
    0x50, 0x57, 0x58, 0x64, 0x65, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x51
    0x4a, 0x4b, 0x53, 0x58, 0x59, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x52
    0x52, 0x59, 0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x53
    0x48, 0x4e, 0x4f, 0x55, 0x57, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x54
    0x4f, 0x54, 0x56, 0x57, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x55
    0x55, 0x57, 0x63, 0x64, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x56
    0x4f, 0x50, 0x51, 0x55, 0x56, 0x63, 0x64, 0x65, 0xff, 0xff, -- 0x57
    0x4a, 0x50, 0x51, 0x52, 0x65, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x58
    0x52, 0x53, 0x66, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x59
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x5f
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x60
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x61
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x62
    0x56, 0x57, 0x64, 0x67, 0x68, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x63
    0x51, 0x56, 0x57, 0x63, 0x65, 0x67, 0x68, 0x69, 0xff, 0xff, -- 0x64
    0x51, 0x57, 0x58, 0x64, 0x68, 0x69, 0xff, 0xff, 0xff, 0xff, -- 0x65
    0x52, 0x59, 0x6a, 0x6b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x66
    0x63, 0x64, 0x68, 0x6c, 0x6d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x67
    0x63, 0x64, 0x65, 0x67, 0x69, 0x6c, 0x6d, 0x6e, 0xff, 0xff, -- 0x68
    0x64, 0x65, 0x68, 0x6a, 0x6d, 0x6e, 0xff, 0xff, 0xff, 0xff, -- 0x69
    0x65, 0x66, 0x69, 0x6b, 0x6e, 0x6f, 0xff, 0xff, 0xff, 0xff, -- 0x6a
    0x66, 0x6a, 0x6f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6b
    0x67, 0x68, 0x6d, 0x71, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6c
    0x67, 0x68, 0x69, 0x6c, 0x6e, 0x71, 0xff, 0xff, 0xff, 0xff, -- 0x6d
    0x68, 0x69, 0x6d, 0x71, 0x72, 0x73, 0xff, 0xff, 0xff, 0xff, -- 0x6e
    0x6a, 0x6b, 0x75, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x6f
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x70
    0x6c, 0x6d, 0x6e, 0x72, 0x77, 0x78, 0xff, 0xff, 0xff, 0xff, -- 0x71
    0x6d, 0x6e, 0x71, 0x73, 0x77, 0x78, 0x79, 0xff, 0xff, 0xff, -- 0x72
    0x6e, 0x72, 0x78, 0x79, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x73
    0x73, 0x75, 0x79, 0x7a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x74
    0x6f, 0x74, 0x7a, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x75
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x76
    0x71, 0x72, 0x78, 0x7c, 0x7d, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x77
    0x71, 0x72, 0x73, 0x77, 0x79, 0x7c, 0x7d, 0x7e, 0xff, 0xff, -- 0x78
    0x72, 0x73, 0x74, 0x78, 0x7a, 0x7d, 0x7e, 0x7f, 0xff, 0xff, -- 0x79
    0x73, 0x74, 0x75, 0x79, 0x7e, 0x7f, 0x80, 0xff, 0xff, 0xff, -- 0x7a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7b
    0x77, 0x78, 0x7d, 0x81, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x7c
    0x77, 0x78, 0x79, 0x7c, 0x7e, 0x81, 0x82, 0xff, 0xff, 0xff, -- 0x7d
    0x78, 0x79, 0x7a, 0x7d, 0x7f, 0x82, 0x83, 0xff, 0xff, 0xff, -- 0x7e
    0x75, 0x79, 0x7a, 0x7e, 0x80, 0x82, 0x83, 0xff, 0xff, 0xff, -- 0x7f
    0x75, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x80
    0x7c, 0x7d, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x81
    0x7c, 0x7d, 0x7e, 0x7f, 0x81, 0x83, 0xff, 0xff, 0xff, 0xff, -- 0x82
    0x7e, 0x7f, 0x80, 0x82, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x83
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x84
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x85
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x86
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x87
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x88
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x89
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8a
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8b
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8c
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8d
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, -- 0x8e
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff -- 0x8f
}
