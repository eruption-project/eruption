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
ticks = 0

-- utility functions --
local function place_raindrop()
    local key = rand(0, num_keys)

    color_map[key] = rgba_to_color(255, 255, 255, lerp(0, 255, opacity))
end

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- let it rain
    if ticks % rand(1, rain_intensity_divisor) == 0 then
        place_raindrop()
    end

    -- fade out raindrops
    if ticks % raindrop_step == 0 then
        for i = 0, num_keys do
            if color_map[i] > 0x00000000 then
                r, g, b, alpha = color_to_rgba(color_map[i])
                alpha = alpha - 10
                color_map[i] = rgba_to_color(r, g, b, max(alpha, 0))

                if color_map[i] < 0x00000000 then
                    color_map[i] = 0x00000000
                end
            end
        end
    end

    submit_color_map(color_map)
end
