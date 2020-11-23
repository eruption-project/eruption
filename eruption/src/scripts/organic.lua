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
require "debug"

-- global state variables --
ticks = 0
color_map = {}

offsets = {0, 0, 0}

function on_mouse_move(rel_x, rel_y, rel_z)
    offsets[1] = offsets[1] - rel_x
    offsets[2] = offsets[2] - rel_y
    offsets[3] = offsets[3] - rel_z
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate the organic effect
    if ticks % animation_delay == 0 then
		local angle = open_simplex_noise_2d(ticks / time_scale, 42)

        for i = 0, canvas_size do
			local x = i / canvas_width
            local y = i / canvas_height

            local x2 = (cos(angle) * x) - (sin(angle) * y)
			local y2 = (sin(angle) * x) + (cos(angle) * y)

            local val = super_simplex_noise((x2 + (offsets[2] / 256)) / coord_scale,
							 			    (y2 + (offsets[1] / 256)) / coord_scale,
                                            (ticks + (offsets[3] / 256)) / time_scale)
            val = lerp(0, 360, val)

            color_map[i] = hsla_to_color((val / color_divisor) + color_offset,
                                         color_saturation, color_lightness,
                                         lerp(0, 255, opacity))
        end

        submit_color_map(color_map)
    end
end
