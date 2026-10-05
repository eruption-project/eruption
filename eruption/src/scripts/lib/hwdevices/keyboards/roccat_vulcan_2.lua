-- SPDX-License-Identifier: GPL-3.0-or-later
-- Experimental German ISO Vulcan II lighting topology.
-- Matrix adapted from OpenRGB, Mola19 (2022), GPL-2.0-or-later;
-- GPLv3-or-later selected for this adaptation.
-- Special Fn/profile/macro HID features remain disabled until verified.

ENABLE_FUNCTION_KEYS = false
ENABLE_MEDIA_KEYS = false
ENABLE_MACRO_KEYS = false
FN_KEY = 255
GAME_MODE_KEY = 255
EASY_SHIFT_KEY = 255
table_offset = 0
num_keys = 108
num_rows = 6
num_cols = 24
max_keys_per_row = num_cols
max_keys_per_col = num_rows
max_neigh = 8

local rows = {
    {0,255,9,15,20,25,255,35,40,45,50,56,62,67,72,255,75,79,84,255,105,106,107,255},
    {1,6,10,16,21,26,30,36,41,46,51,57,63,68,255,255,76,80,85,255,88,93,97,102},
    {2,255,11,17,22,27,31,37,42,47,52,58,64,69,255,255,77,81,86,255,89,94,98,103},
    {3,255,12,18,23,28,32,38,43,48,53,59,65,70,73,255,255,255,255,255,90,95,99,255},
    {4,7,13,19,24,29,33,39,44,49,54,60,255,71,255,255,255,82,255,255,91,96,100,104},
    {5,8,14,255,255,255,34,255,255,255,55,61,66,255,74,255,78,83,87,255,92,255,101,255},
}

-- Names describe German keycaps; table values are one-based Lua indices.
local names = {
    'ESC','^','TAB','CAPS_LOCK','LEFT_SHIFT','LEFT_CTRL','1','<','MOD_LEFT',
    'F1','2','Q','A','Y','LEFT_ALT','F2','3','W','S','X','F3','4','E','D','C',
    'F4','5','R','F','V','6','T','G','B','SPACE','F5','7','Z','H','N',
    'F6','8','U','J','M','F7','9','I','K',',','F8','0','O','L','.',
    'RIGHT_ALT','F9','ß','P','Ö','-','FN','F10','´','Ü','Ä','RIGHT_MENU',
    'F11','BACKSPACE','+','#','RIGHT_SHIFT','F12','RETURN','RIGHT_CTRL',
    'PRINT','INSERT','DEL','LEFT','ROLL','POS1','END','UP','DOWN','PAUSE',
    'PGUP','PGDWN','RIGHT','NUM','NUM_7','NUM_4','NUM_1','NUM_0','NUM_DIV',
    'NUM_8','NUM_5','NUM_2','NUM_MULT','NUM_9','NUM_6','NUM_3','NUM_COMMA',
    'NUM_MINUS','NUM_PLUS','NUM_RETURN','MEDIA_PREVIOUS','MEDIA_PLAY_PAUSE','MEDIA_NEXT',
}
key_to_index = {}
for index, name in ipairs(names) do key_to_index[name] = index end

rows_topology = {}
cols_topology = {}
coordinates_to_index = {}
keys_per_col = {}
neighbor_topology = {}

for row = 1, num_rows do
    for col = 1, num_cols do
        local key = rows[row][col]
        rows_topology[(row - 1) * num_cols + col] = key
        coordinates_to_index[(row - 1) * num_cols + col] = key == 255 and 255 or key + 1
    end
end
for col = 1, num_cols do
    keys_per_col[col] = 0
    for row = 1, num_rows do
        local key = rows[row][col]
        cols_topology[(col - 1) * num_rows + row] = key
        if key ~= 255 then keys_per_col[col] = keys_per_col[col] + 1 end
    end
end

-- Nearest matrix key along each of eight directions; no edge wraparound.
-- This approximate neighbor model is not a physical-distance calibration.
local directions = {{-1,0},{1,0},{0,-1},{0,1},{-1,-1},{-1,1},{1,-1},{1,1}}
for row = 1, num_rows do
    for col = 1, num_cols do
        local key = rows[row][col]
        if key ~= 255 then
            for slot, direction in ipairs(directions) do
                local r, c = row + direction[1], col + direction[2]
                local neighbor = 255
                while r >= 1 and r <= num_rows and c >= 1 and c <= num_cols do
                    if rows[r][c] ~= 255 then neighbor = rows[r][c]; break end
                    r, c = r + direction[1], c + direction[2]
                end
                neighbor_topology[key * max_neigh + slot] = neighbor
            end
        end
    end
end

function device_specific_key_highlights() end
function device_specific_key_highlights_indicators() end
