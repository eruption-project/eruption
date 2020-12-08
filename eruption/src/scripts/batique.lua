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

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate batique effect
    if ticks % animation_delay == 0 then
        for i = zone_start, zone_end do
            local x = i / (zone_end - zone_start)
            local y = i / (zone_end - zone_start)

            local val = super_simplex_noise((x / coord_scale),
                                            (y / coord_scale),
                                            ticks / time_scale)
            val = lerp(0, 360, val)

            color_map[i] = hsla_to_color((val / color_divisor) + color_offset,
                                            color_saturation, color_lightness,
                                            lerp(0, 255, opacity))
        end

        submit_color_map(color_map)
    end
end
