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

require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_tick(delta)
    ticks = ticks + delta

    if horizontal then
        for i = num_cols, 0, -1 do
            for j = 1, max_keys_per_col do
                local alpha = (sin(i / wave_length + (ticks * direction / speed_divisor)) + 1) * scale_factor
                local r, g, b = color_to_rgb(color_wave)

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = rgba_to_color(r, g, b, clamp(0, 255, alpha * opacity))
            end
        end
    else
        for i = num_rows, 0, -1 do
            for j = 1, max_keys_per_row do
                local alpha = (sin(i / wave_length + (ticks * direction / speed_divisor)) + 1) * scale_factor
                local r, g, b = color_to_rgb(color_wave)

                local index = n(rows_topology[j + (i * max_keys_per_row)]) + 1
                color_map[index] = rgba_to_color(r, g, b, clamp(0, 255, alpha * opacity))
            end
        end
    end

    submit_color_map(color_map)
end
