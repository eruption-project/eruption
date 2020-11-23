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
max_effect_ttl = target_fps * 3
effect_ttl = max_effect_ttl

function on_startup(config)
	for i = 0, canvas_size do
		r, g, b, alpha = color_to_rgba(color_background)
		color_map[i] = rgba_to_color(r, g, b, lerp(0, 255, opacity))
	end

	submit_color_map(color_map)
end

function on_tick(delta)
	if effect_ttl <= 0 then return end

	effect_ttl = effect_ttl - 1

	submit_color_map(color_map)
end
