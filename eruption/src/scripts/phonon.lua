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
require "utilities"
require "debug"

-- global state variables --
color_map = {}
max_effect_ttl = target_fps * 2
effect_ttl = max_effect_ttl

-- holds a scalar field to simulate a wave
grid = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end

    -- initialize scalar field
    for i = 0, num_keys do
        grid[i] = 0.0
    end
end

function on_key_up(key_index)
    if key_index ~= 0 then
        for i = 0, max_neigh do
            local neigh_key = n(neighbor_topology[(key_index * max_neigh) + i + table_offset]) + 1

            if neigh_key ~= 0xff then
                grid[neigh_key] = 0.5
            end
        end
    end

	effect_ttl = max_effect_ttl
end

function on_tick(delta)
	ticks = ticks + delta

	if effect_ttl <= 0 then return end

    if ticks % flow_speed == 0 then
        -- compute phonon effect
        for key_index = 1, num_keys do
            local epsilon = 0.1
            if grid[key_index] >= epsilon then
                grid[key_index - 1] = grid[key_index] - 0.1
            else
                grid[key_index - 1] = 0.0
            end

            -- compute color
            color_map[key_index] = hsla_to_color(lerp(120, 220, sin(grid[key_index])) + 1.25, 1.0, 0.5,
																	lerp(0, 255, sin(grid[key_index])))
		end

		effect_ttl = effect_ttl - 1

		submit_color_map(color_map)
    end
end
