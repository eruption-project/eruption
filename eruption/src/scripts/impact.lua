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
-- along with Eruption.  If not, see <http:--www.gnu.org/licenses/>.

-------------------------------------------------------------------------------
-- This script was heavily inspired by the excellent work of duncanthrax
-- Please see: https://github.com/duncanthrax/roccat-vulcan
-------------------------------------------------------------------------------

require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
ticks = 0
max_effect_ttl = target_fps * 3
effect_ttl = max_effect_ttl

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
		color_map[i] = 0x00000000
	end
end

function on_key_down(key_index)
	color_map[key_index] = color_impact

	if key_index ~= 0 then
		for i = 0, max_neigh do
			local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset]) + 1

			if neigh_key ~= 0xff then
				color_map[neigh_key] = color_impact
			end
		end
	end

	effect_ttl = max_effect_ttl
end

function on_tick(delta)
	ticks = ticks + delta

	if effect_ttl <= 0 then return end

	-- compute impact effect
	if ticks % impact_step == 0 then
		for i = 0, canvas_size do
			color = color_map[i]
			if color ~= nil then
				r, g, b, alpha = color_to_rgba(color)
				color_map[i] = rgba_to_color(r, g, b, max(alpha - alpha_step_impact, 0))

				if alpha < 1 then
					color_map[i] = 0x00000000
				end
			end
		end

		effect_ttl = effect_ttl - 1

		submit_color_map(color_map)
	end
end
