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
color_map = {}
color_map_pressed = {}

ticks = 0
adaptive_offset = 0

-- event handler functions --
function on_startup(config)
    local num_keys = get_num_keys()

    for i = 0, num_keys do
        color_map[i] = color_off
    end
end

function on_tick(delta)
    ticks = ticks + delta

    adaptive_offset = get_audio_loudness() / 50000
    -- debug("AudioViz5: Adaptive offset: " .. adaptive_offset)

    -- calculate batique effect
    if ticks % animation_delay == 0 then
        local num_keys = get_num_keys()

        for i = 0, num_keys do
            local x = i / num_rows
            local y = i / num_cols

            local val = open_simplex_noise(x / coord_scale + adaptive_offset,
                                           y / coord_scale + adaptive_offset,
                                           ticks / time_scale)
            val = lerp(0, 360, val)

            color_map[i] = hsla_to_color((val / color_divisor) + color_offset,
                                            color_saturation, color_lightness,
                                            lerp(0, 255, opacity))
        end

        submit_color_map(color_map)
    end
end
