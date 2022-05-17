-- This file is part of Eruption.
--
-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
--
-- Copyright (c) 2019-2022, The Eruption Development Team
--
require "declarations"
require "debug"

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    -- static gradient
    if not animate_gradient then
        for i = 0, canvas_size - 1 do
            color_map[i + 1] = linear_gradient(color_start, color_end,
                                               i / canvas_size)
        end

        submit_color_map(color_map)
    end
end

function on_apply_parameter(parameter, value)
    local update_fn = load("" .. parameter .. " = " .. value)

    update_fn()

    -- update state
    on_startup(nil)
end

function on_tick(delta)
    if not animate_gradient then return end

    ticks = ticks + delta

    -- animate gradient
    if ticks % gradient_step == 0 then
        for i = 0, canvas_size - 1 do
            local p = ((i + ticks) / color_divisor) % 100
            color_map[i + 1] = linear_gradient(color_start, color_end, p / 100)
        end

        submit_color_map(color_map)
    end
end
