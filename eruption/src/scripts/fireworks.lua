-- SPDX-License-Identifier: GPL-3.0-or-later
--
-- This file is part of Eruption.
--
-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
--
-- Copyright (c) 2019-2022, The Eruption Development Team
--
require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
max_effect_ttl = target_fps * 2
effect_ttl = max_effect_ttl

-- holds a scalar field to simulate fireworks
fireworks_grid = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    for i = 1, num_keys do fireworks_grid[i] = 0.0 end
end

function on_key_down(key_index) effect_ttl = max_effect_ttl end

function on_key_up(key_index) effect_ttl = max_effect_ttl end

local function update_key_states()
    for key_index = 1, num_keys do
        local pressed = get_key_state(key_index)

        if pressed then
            for i = 1, max_neigh do
                local neigh_key = n(neighbor_topology[(key_index * max_neigh) +
                                        i + table_offset]) + 1

                if neigh_key ~= 0xff then
                    fireworks_grid[neigh_key] = 1.0
                end
            end
        end
    end
end

function on_tick(delta)
    ticks = ticks + delta

    if effect_ttl <= 0 then return end

    update_key_states()

    -- calculate fireworks effect
    if ticks % animation_speed == 0 then
        -- compute fireworks effect
        for key_index = 1, num_keys - 1 do
            local avg = n((fireworks_grid[key_index - 1] +
                              fireworks_grid[key_index + 1]) / 2)
            fireworks_grid[key_index - 1] =
                (fireworks_grid[key_index] - 0.25) + (avg * 0.5)

            local epsilon = 0.1
            if fireworks_grid[key_index] <= epsilon or fireworks_grid[key_index] >=
                (1.0 - epsilon) then fireworks_grid[key_index] = 0.0 end
        end

        for y = 0, num_rows - 1 do
            for x = 0, max_keys_per_row - 1 do
                local idx = y * max_keys_per_row + x

                if n(fireworks_grid[idx]) > 0 then
                    fireworks_grid[idx] =
                        fireworks_grid[idx] - (fireworks_grid[idx] * 0.25)
                end

                local epsilon = 0.1
                if n(fireworks_grid[idx]) <= epsilon then
                    fireworks_grid[idx] = 0.0
                end

                -- compute color
                if n(fireworks_grid[idx]) > 0.0 then
                    local hue = lerp(0, 360, sin(fireworks_grid[idx]))
                    color_map[idx] = hsla_to_color(hue, 1.0, 0.5,
                                                   lerp(0, 255, opacity))
                else
                    color_map[idx] = 0x00000000
                end
            end
        end

        effect_ttl = effect_ttl - 1

        submit_color_map(color_map)
    end
end
