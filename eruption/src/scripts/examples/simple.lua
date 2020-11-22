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

-- global array that stores each key's current color
color_map = {}

function on_startup()
    -- turn off all LEDs
    for i = 0, get_canvas_size() do
        color_map[i] = 0x00000000
    end

    -- update LED state
    submit_color_map(color_map)
end

function on_key_down(key_index)
    info("Pressed key: " .. key_index)

    -- set color of pressed key to red
    color_map[key_index] = rgba_to_color(255, 0, 0, 255)
    submit_color_map(color_map)
end
