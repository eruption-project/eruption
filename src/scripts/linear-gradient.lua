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

-- please find example gradients below

-- repeating linear red to light cold white gradient
-- color_start = rgb_to_color(255, 0, 0)
-- color_end = rgb_to_color(0, 255, 255)
-- color_divisor = 128
-- animate_gradient = false
-- gradient_speed = 1

-- red to light cold white gradient
-- color_start = rgb_to_color(255, 0, 0)
-- color_end = rgb_to_color(0, 255, 255)
-- color_divisor = 256
-- animate_gradient = false
-- gradient_speed = 1

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        color_map[i] = rgba_to_color(0, 0, 0, 0)
    end
end

function on_tick(delta)
    ticks = ticks + delta + 1

    local num_keys = get_num_keys()

    -- animate gradient
    if animate_gradient and (ticks % gradient_step == 0) then
        for i = 0, num_keys do
            color_map[i] = linear_gradient(color_start, color_end, ((i + ticks) / color_divisor) * gradient_speed)
        end
    end

    submit_color_map(color_map)
end
