-- This file is part of Eruption.

-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.

-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.

-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

-- target framerate of the core engine
target_fps = get_target_fps()

-- canvas dimensions
canvas_size = get_canvas_size()
canvas_height = get_canvas_height()
canvas_width = get_canvas_width()

keyboard_zone_start = 0
keyboard_zone_end = get_num_keys()

mouse_zone_start = 144
mouse_zone_end = 144 + 36

-- Keyboard topology maps --
-- use 'table_offset = 0' for the ISO model
-- table_offset = get_num_keys() + 1
table_offset = 0

-- Load support scripts that contain hardware specific declarations
local function load_support_scripts()
	local support_files = get_support_script_files()

	for k, file in pairs(support_files) do
		debug("Loading device specific Lua script: '" .. file .. ".lua'")

		require("hwdevices/" .. file)
	end
end

load_support_scripts()
