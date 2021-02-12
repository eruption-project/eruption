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
ticks = 0
color_map = {}
offsets = {0, 0, 0}

grad = gradient_from_name(stock_gradient)

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_mouse_move(rel_x, rel_y, rel_z)
    offsets[1] = offsets[1] - rel_x
    offsets[2] = offsets[2] - rel_y
    offsets[3] = offsets[3] - rel_z
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate the lava lamp effect
    if ticks % animation_delay == 0 then
        for i = num_rows, 0, -1 do
            for j = 1, max_keys_per_row do
                local x = j / coord_scale
                local y = i / coord_scale

                local val = perlin_noise((x + (offsets[1] / 2048)),
                                         (y + (offsets[2] / 2048)),
                                         ticks / time_scale)

                local pos = clamp(val + 0.5 / 2.0, 0.0, 1.0)

                local index = n(rows_topology[j + (i * max_keys_per_row)]) + 1
                color_map[index] = gradient_color_at(grad, pos)
            end
        end

        submit_color_map(color_map)
    end
end
