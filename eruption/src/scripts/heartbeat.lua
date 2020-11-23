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
require "easing"

-- global state variables --
heartbeat_step = 1.25
color_map = {}
ticks = 0
percentage = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- update system load indicator approximately every second
    if ticks % target_fps == 0 then
        heartbeat_step = max(min(get_runnable_tasks() * heartbeat_multiplier, 3.25), 0.25)
        trace("HeartBeat: Runqueue: " .. get_runnable_tasks() .. " Step: " .. heartbeat_step)
    end

    -- calculate 'fill' percentage for heartbeat effect
    percentage = percentage + ((heartbeat_step * max(delta, 1)) + (easing(percentage) * heartbeat_step))
    if percentage >= (100 - heartbeat_upper_lim) then
        percentage = 100 - heartbeat_upper_lim
        heartbeat_step = heartbeat_step * -1
    elseif percentage <= (0 + heartbeat_lower_lim) then
        percentage = 0 + heartbeat_lower_lim
        heartbeat_step = heartbeat_step * -1
    end

    -- generate heartbeat color map values
    local upper_bound = num_keys * (clamp(percentage, 0, 100) / 100)
    for i = 0, num_keys do
        if i <= upper_bound then
            color_map[i] = color_map[i] + rgba_to_color(0, 0, 0, 10)

            if color_map[i] >= 0xffffffff then
                color_map[i] = 0xffffffff
            elseif color_map[i] <= 0x00000000 then
                color_map[i] = 0x00000000
            end
        else
            color_map[i] = 0x00000000
        end
    end

    submit_color_map(color_map)
end

-- a simple easing function that mimics heartbeat
function easing(x)
    return pow(sin(5 * x / 3.14159), 2)
end
