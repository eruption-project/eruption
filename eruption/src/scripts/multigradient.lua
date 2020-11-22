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

-- set gradient stops
gradient_stops = {
    [0] = { start = rgba_to_color(255,   0,   0, lerp(0, 255, opacity)), dest = rgba_to_color(255, 165,   0, lerp(0, 255, opacity)) },
    [1] = { start = rgba_to_color(255, 165,   0, lerp(0, 255, opacity)), dest = rgba_to_color(0,   255, 255, lerp(0, 255, opacity)) },
    [2] = { start = rgba_to_color(0,   255, 255, lerp(0, 255, opacity)), dest = rgba_to_color(0,   255,   0, lerp(0, 255, opacity)) },
    [3] = { start = rgba_to_color(0,   255,   0, lerp(0, 255, opacity)), dest = rgba_to_color(0,     0, 255, lerp(0, 255, opacity)) },
    [4] = { start = rgba_to_color(0,     0, 255, lerp(0, 255, opacity)), dest = rgba_to_color(75,    0, 130, lerp(0, 255, opacity)) },
    [5] = { start = rgba_to_color(75,    0, 130, lerp(0, 255, opacity)), dest = rgba_to_color(238, 130, 238, lerp(0, 255, opacity)) },
    [6] = { start = rgba_to_color(238, 130, 238, lerp(0, 255, opacity)), dest = rgba_to_color(255,   0,   0, lerp(0, 255, opacity)) },
    len = 7
}

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = linear_gradient_multi(gradient_stops, (i * num_keys / 100))
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- animate gradient
    if animate_gradient and (ticks % gradient_step == 0) then
        for i = 0, canvas_size do
            color_map[i] = linear_gradient_multi(gradient_stops, i + ticks)
        end
    end

    submit_color_map(color_map)
end

-- support functions
function linear_gradient_multi(stops, p)
    local i = trunc(clamp(p / (100 * stops.len), 0, stops.len - 1))

    -- info("p: " .. p .. " " .. "index: " .. i)

    local s = stops[i].start
    local e = stops[i].dest

    local result = linear_gradient(s, e, p / color_divisor)

    -- info("result: " .. result)

    return result
end
