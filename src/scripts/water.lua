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
max_effect_ttl = 40

effect_ttl = 0

-- holds a scalar field to simulate water
water_grid = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        color_map[i] = rgba_to_color(0, 0, 0, 0)
    end

    -- initialize water scalar field
    for i = 0, num_keys do
        water_grid[i] = 0.0
    end
end

function on_key_up(key_index)
    for i = 0, max_neigh do
		local neigh_key = neighbor_topology[(key_index * max_neigh) + i + table_offset] + 1

		if neigh_key ~= 0xff then
			water_grid[neigh_key] = 0.5
		end
	end
	
	effect_ttl = max_effect_ttl
end

function on_tick(delta)
	ticks = ticks + delta + 1
	
	if effect_ttl <= 0 then return end

	local num_keys = get_num_keys()

    if ticks % flow_speed == 0 then
        -- compute wave effect
        for key_index = 1, get_num_keys() do
            local epsilon = 0.1
            if water_grid[key_index] >= epsilon then
                water_grid[key_index - 1] = water_grid[key_index] - 0.1
            else
                water_grid[key_index - 1] = 0.0
            end

            -- compute color
            color_map[key_index] = hsla_to_color(lerp(120, 220, sin(water_grid[key_index])) + 1.25, 1.0, 0.5,
																	lerp(0, 255, sin(water_grid[key_index])))
		end
		
		effect_ttl = effect_ttl - 1

		submit_color_map(color_map)
    end
end
