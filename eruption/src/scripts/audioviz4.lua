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
percentage = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate 'fill' percentage for heartbeat effect
    percentage = min((get_audio_loudness() / 100) * loudness_scale_factor, 100.0)

    -- generate heartbeat color map values
    local upper_bound = num_keys * (min(percentage, 100) / 100)
    for i = 0, num_keys do
        if i <= upper_bound then
            color_map[i] = color_map[i] + color_step

            if color_map[i] >= 0xffffffff then
                color_map[i] = rgba_to_color(128, 64, 255, lerp(0, 255, opacity))
            elseif color_map[i] <= 0xff000000 then
                color_map[i] = 0xff000000
            end
        else
            color_map[i] = 0xffffffff
        end
    end

    submit_color_map(color_map)
end
