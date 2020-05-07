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

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        color_map[i] = rgba_to_color(0, 0, 0, 0)
    end


    -- highlight WASD keys
    color_map[9]  = color_highlight
    color_map[14] = color_highlight
    color_map[15] = color_highlight
    color_map[21] = color_highlight

    submit_color_map(color_map)
end

-- function on_tick(delta)
--     ticks = ticks + delta + 1
--     submit_color_map(color_map)
-- end
