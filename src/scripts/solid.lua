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

function on_tick(delta)
    local num_keys = get_num_keys()
    for i = 0, num_keys do
				r, g, b, alpha = color_to_rgba(color_background)
        color_map[i] = rgba_to_color(r, g, b, lerp(0, 255, opacity))
    end

		submit_color_map(color_map)
end
