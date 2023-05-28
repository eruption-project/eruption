-- SPDX-License-Identifier: GPL-3.0-or-later
--
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
-- Copyright (c) 2019-2023, The Eruption Development Team
--
require "declarations"
require "debug"

-- global state variables --
ticks = 0
color_map = {}

grad = gradient_from_name(stock_gradient)

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do color_map[i] = 0x00000000 end
end

function on_render() submit_color_map(color_map) end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate batique effect
    if ticks % animation_delay == 0 then
        local zone = {x = 0, y = 0, x2 = canvas_width, y2 = canvas_height}
        if use_zone_index then zone = zones[zone_index] end

        for y = zone.y, zone.y2 - 1 do
            for x = zone.x, zone.x2 - 1 do
                local index = (y * canvas_width) + x + 1

                local val = super_simplex_noise((x / coord_scale),
                                                (y / coord_scale),
                                                ticks / time_scale)

                if use_gradient then
                    color_map[index] = gradient_color_at(grad, val)
                else
                    color_map[index] = hsla_to_color(
                                           (val / color_divisor) + color_offset,
                                           1.0, 0.5, lerp(0, 255, opacity))
                end
            end
        end
    end
end
