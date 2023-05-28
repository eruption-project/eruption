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

-- event handler functions --
function on_startup(config)
    -- clear the color map
    for i = 0, canvas_size do color_map[i] = 0x00000000 end

    -- dim zone 'zone_index'
    local zone = zones[zone_index]
    for y = zone.y, zone.y2 - 1 do
        for x = zone.x, zone.x2 - 1 do
            local index = (y * canvas_width) + x + 1
            color_map[index] = rgba_to_color(0, 0, 0, 255 * opacity)
        end
    end

    submit_color_map(color_map)
end

function on_apply_parameter(parameters)
    -- update state
    on_startup(nil)
end

-- function on_render()
--     submit_color_map(color_map)
-- end
