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
state_map = {}
visited_map = {}
color_map = {}
color_map_afterglow = {}
ticks = 0

-- utility functions --
local function set_neighbor_states(key_index, value)
	if key_index ~= nil and key_index ~= 0 then
		for i = 0, max_neigh do
			local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset])

			if neigh_key ~= nil and neigh_key ~= 0xff then
				state_map[neigh_key + 1] = value
				visited_map[neigh_key + 1] = true
			end
		end
	end
end

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
		state_map[i] = key_state.idle
		color_map[i] = 0x00000000
		color_map_afterglow[i] = 0x00000000
    end
end

function on_key_down(key_index)
	color_map_afterglow[key_index] = color_afterglow
 	set_neighbor_states(key_index, key_state.shockwave_origin)

	effect_ttl = max_effect_ttl
end

function on_mouse_button_down(button_index)
	if not mouse_events then return end

	for i = 0, canvas_size do
		color_map[i] = color_mouse_click_flash
	end

	effect_ttl = max_effect_ttl
end

function on_mouse_button_up(button_index)
	if not mouse_events then return end

	for i = 0, canvas_size do
		color_map[i] = color_mouse_click_flash
	end

	effect_ttl = max_effect_ttl
end

function on_mouse_wheel(direction)
	if not mouse_events then return end

	if direction == 1 then
		c = color_mouse_wheel_flash
	elseif direction == 2 then
		c = color_mouse_wheel_flash
	end

	for i = 0, canvas_size do
		color_map[i] = c
	end

	effect_ttl = max_effect_ttl
end

function on_mouse_hid_event(event_type, arg1)
	if not mouse_events then return end

	if event_type == 1 then
		-- DPI change event
		for i = 0, canvas_size do
			color_map[i] = color_mouse_wheel_flash
		end

		effect_ttl = max_effect_ttl
	end
end

function on_tick(delta)
	ticks = ticks + delta

	if effect_ttl <= 0 then return end

	for i = 0, canvas_size do
		visited_map[i] = false
	end

	-- propagate the shockwave
	for i = 1, canvas_size do
		-- decrease key ttl
		if state_map[i] > key_state.shockwave_sentinel then
			state_map[i] = state_map[i] - shockwave_ttl_decrease
			if state_map[i] <= key_state.shockwave_sentinel then
				state_map[i] = key_state.idle
			end
		end

		-- propagate wave effect
		if not visited_map[i] and state_map[i] >= key_state.shockwave_sentinel then
			local neigh_key = n(neighbor_topology[(i * max_neigh) + 0 + table_offset]) + 1

			if neigh_key ~= nil and neigh_key ~= 0xff then
				state_map[neigh_key] = state_map[i] - shockwave_ttl_decrease
				visited_map[neigh_key] = true

				set_neighbor_states(i, state_map[neigh_key])
			end
		else
			local neigh_key = n(neighbor_topology[(i * max_neigh) + 0 + table_offset]) + 1

			if neigh_key ~= 0xff then
				state_map[neigh_key] = key_state.idle
			end
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

		-- compute afterglow
		if color_map_afterglow[i] > 0x00000000 then
			color_map_afterglow[i] = color_map_afterglow[i] - color_step_afterglow
			color_map[i] = color_map_afterglow[i]
		else
			color_map_afterglow[i] = 0x00000000
		end

		-- safety net
		if color_map[i] == nil or
		   color_map[i] < 0x00000000 or color_map[i] > 0xffffffff then
			color_map[i] = 0x00000000
		end
	end

	effect_ttl = effect_ttl - 1

	submit_color_map(color_map)
end
