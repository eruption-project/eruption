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
require "utilities"
require "debug"

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    -- highlight WASD keys
    color_map[key_name_to_index("W")] = color_highlight
    color_map[key_name_to_index("A")] = color_highlight
    color_map[key_name_to_index("S")] = color_highlight
    color_map[key_name_to_index("D")] = color_highlight

    submit_color_map(color_map)
end

function on_apply_parameter(parameters)
    -- update state
    on_startup(nil)
end

-- function on_tick(delta)
--     ticks = ticks + delta
--     submit_color_map(color_map)
-- end
