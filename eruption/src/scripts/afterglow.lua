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
color_map_glow = {}
ticks = 0
max_effect_ttl = target_fps * 8
effect_ttl = max_effect_ttl

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
		color_map[i] = 0x00000000
	end
end

function on_key_down(key_index)
   color_map[key_index] = color_afterglow

   effect_ttl = max_effect_ttl
end

function on_tick(delta)
	ticks = ticks + delta

	if effect_ttl <= 0 then return end

    -- calculate afterglow effect for pressed keys
    if ticks % afterglow_step == 0 then
		for i = 0, canvas_size do
			r, g, b, alpha = color_to_rgba(color_map[i])
			if alpha > 0 then
				color_map[i] = rgba_to_color(r, g, b, max(alpha - alpha_step_afterglow, 0))
			else
				color_map[i] = 0x00000000
			end
		end

		effect_ttl = effect_ttl - 1

		submit_color_map(color_map)
    end
end
