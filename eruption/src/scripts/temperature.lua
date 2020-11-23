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
temperature = get_package_temp()
max_temperature = get_package_max_temp()
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    percentage = 0

    for i = 0, canvas_size do
        color_map[i] = color_background
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- update the temperature
    if ticks % 5 == 0 then
        temperature = get_package_temp()
        trace("Temperature: Temp: " .. get_package_temp() .. " / " .. max_temperature)
    end

    -- calculate colors
    local percentage = min(temperature / max_temperature * 100, 100)

    for i = 0, canvas_size do
        color_map[i] = linear_gradient(color_cold, color_hot, percentage / 100)
    end

    submit_color_map(color_map)
end
