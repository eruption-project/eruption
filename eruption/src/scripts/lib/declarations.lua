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

-- target framerate of the core engine
target_fps = get_target_fps()

-- canvas dimensions
canvas_size = get_canvas_size()
canvas_height = get_canvas_height()
canvas_width = get_canvas_width()

keyboard_zone_start = 1
keyboard_zone_end = get_num_keys()

mouse_zone_start = get_canvas_size() - 36
mouse_zone_end = get_canvas_size() + 36

-- Keyboard topology maps --
-- use 'table_offset = 0' for the ISO model
-- table_offset = get_num_keys() + 1
table_offset = 0

-- character to key index mapping
key_to_index = {}

-- coordinates to key index mapping
coordinates_to_index = {}

keys_per_col = {}

num_keys = 0

-- rows
num_rows = 0
max_keys_per_row = 0
rows_topology = {}

-- columns
num_cols = 0
max_keys_per_col = 0
cols_topology = {}

-- neighbor tables
max_neigh = 0
neighbor_topology = {}

-- support functions
function device_specific_key_highlights()
    -- empty stub
end

function device_specific_key_highlights_indicators()
    -- empty stub
end

-- Load support scripts that contain hardware specific declarations
local function load_support_scripts()
    local support_files = get_support_script_files()

    for k, file in pairs(support_files) do
        debug("Loading device specific Lua script: '" .. file .. ".lua'")

        local status, l = pcall(require, "hwdevices/" .. file)
        if not status then
            error("Could not load device specific script: " .. file .. ".lua")
        end
    end
end

load_support_scripts()
