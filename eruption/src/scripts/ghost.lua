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

key_state = {
	invalid = 0,
	idle = 1,

	shockwave_sentinel = 16,
	shockwave_origin = 32,
}

max_effect_ttl = target_fps * 2
effect_ttl = max_effect_ttl

-- max ttl of a shockwave cell
shockwave_ttl = key_state.shockwave_origin - key_state.shockwave_sentinel
shockwave_ttl_decrease = (key_state.shockwave_origin - key_state.shockwave_sentinel) / shockwave_divisor

-- global state variables --
color_map = {}
state_map = {}
ticks = 0
next_rand = ghost_backoff_secs * target_fps

-- utility functions --
local function ghost_key(key_index)
	color_map[key_index] = color_afterglow

	if key_index ~= 0 then
		for i = 0, max_neigh do
			local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset]) + 1

			if neigh_key ~= 0xff then
				state_map[neigh_key] = key_state.shockwave_origin
			end
		end
	end

	effect_ttl = max_effect_ttl
end

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
        color_map[i] = 0x00000000
	end

    for i = 0, num_keys do
		state_map[i] = key_state.idle
    end
end

function on_key_down(key_index)
	next_rand = ticks + (ghost_backoff_secs * target_fps)
end

function on_tick(delta)
	ticks = ticks + delta

	if ticks > next_rand then
		ghost_key(trunc(rand(1, num_keys)))
		next_rand = ticks + trunc(rand(0.2 / ghost_intensity * target_fps,
										 4 / ghost_intensity * target_fps))
	end

	if effect_ttl <= 0 then return end

	-- propagate the shockwave
	for i = 1, num_keys do
		-- decrease key ttl
		if state_map[i] > key_state.shockwave_sentinel then
			state_map[i] = state_map[i] - shockwave_ttl_decrease
			if state_map[i] <= key_state.shockwave_sentinel then
				state_map[i] = key_state.idle
			end

		end

		-- propagate wave effect
		if state_map[i] >= key_state.shockwave_sentinel then
			state_map[i - 1] = state_map[i] - shockwave_ttl_decrease
		else
			state_map[i - 1] = key_state.idle
		end

		-- compute shockwave color
		if state_map[i] >= key_state.shockwave_sentinel then
			color_map[i] = color_shockwave - color_step_shockwave
		end

		if color_map[i] > 0x00000000 then
			color_map[i] = color_map[i] - color_step_shockwave

			if color_map[i] < 0x00000000 then
				color_map[i] = 0x00000000
			end
		end
	end

	effect_ttl = effect_ttl - 1

	submit_color_map(color_map)
end
