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

require "declarations"
require "debug"

key_state = {
	invalid = 0,
	idle = 1,

	water_sentinel = 16,
	water_origin = 255,
}

max_effect_ttl = target_fps * 10
effect_ttl = max_effect_ttl

-- max ttl of a water cell
water_ttl = key_state.water_origin - key_state.water_sentinel
water_ttl_decrease = (key_state.water_origin - key_state.water_sentinel) / wave_divisor

-- global state variables --
color_map = {}
state_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, num_keys do
        color_map[i] = 0x00000000
		state_map[i] = key_state.idle
    end
end

function on_key_down(key_index)
	color_map[key_index] = color_water

	if key_index ~= 0 then
		for i = 0, max_neigh do
			local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset]) + 1

			if neigh_key ~= 0xff then
				state_map[neigh_key] = key_state.water_origin
			end
		end
	end

	effect_ttl = max_effect_ttl
end

function on_key_up(key_index)
	color_map[key_index] = color_water

	if key_index ~= 0 then
		for i = 0, max_neigh do
			local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset]) + 1

			if neigh_key ~= 0xff then
				state_map[neigh_key] = key_state.water_origin
			end
		end
	end

	effect_ttl = max_effect_ttl
end

function on_tick(delta)
	ticks = ticks + delta

	if effect_ttl <= 0 then return end

	-- propagate the water wave
	for i = 1, num_keys do
		-- decrease key ttl
		if state_map[i] > key_state.water_sentinel then
			state_map[i] = state_map[i] - 1
			if state_map[i] <= key_state.water_sentinel then
				state_map[i] = key_state.idle
			end
		end

		if ticks % 1 == 0 then
			-- propagate wave effect
			if state_map[i] >= key_state.water_sentinel then
				state_map[i - 1] = state_map[i] - 15
			else
				state_map[i - 1] = key_state.idle
			end
		end

		-- compute water color
		if state_map[i] >= key_state.water_sentinel then
			color_map[i] = hsla_to_color(state_map[i], 0.5, 0.5, 255)
		else
			color_map[i] = 0x00000000
		end
	end

	effect_ttl = effect_ttl - 1

	submit_color_map(color_map)
end
