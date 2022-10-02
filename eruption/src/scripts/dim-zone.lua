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
ticks = 0
color_map = {}

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    for i = zone_start, zone_end do
        color_map[i] = rgba_to_color(0, 0, 0, 255 * opacity)
    end

    submit_color_map(color_map)
end

function on_apply_parameter(parameter, value)
    -- update state
    on_startup(nil)
end
