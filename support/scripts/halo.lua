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
-- Copyright (c) 2019-2023, The Eruption Development Team
--
require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
color_map_afterglow = {}
color_map_effects = {}
max_effect_ttl = target_fps * 2
effect_ttl = max_effect_ttl

-- holds a scalar field to simulate a wave
grid = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
        color_map_afterglow[i] = 0x00000000
        color_map_effects[i] = 0x00000000
        grid[i] = 0.0
    end
end

function on_mouse_button_down(button_index)
    if not mouse_events then return end

    for i = 0, canvas_size do color_map_effects[i] = color_mouse_click_flash end

    effect_ttl = max_effect_ttl
end

function on_mouse_button_up(button_index)
    if not mouse_events then return end

    for i = 0, canvas_size do color_map_effects[i] = 0x00000000 end

    effect_ttl = max_effect_ttl
end

function on_mouse_wheel(direction)
    if not mouse_events then return end

    if direction == 1 then
        c = color_mouse_wheel_flash
    elseif direction == 2 then
        c = color_mouse_wheel_flash
    elseif direction == 3 then
        c = color_mouse_wheel_flash
    elseif direction == 4 then
        c = color_mouse_wheel_flash
    end

    for i = 0, canvas_size do color_map_effects[i] = c end

    effect_ttl = max_effect_ttl
end

function on_mouse_hid_event(event_type, arg1)
    if not mouse_events then return end

    if event_type == 1 then
        -- DPI change event
        for i = 0, canvas_size do
            color_map_effects[i] = color_mouse_wheel_flash
        end

        effect_ttl = max_effect_ttl
    end
end

function on_key_down(key_index) effect_ttl = max_effect_ttl end

function on_key_up(key_index) effect_ttl = max_effect_ttl end

local function update_key_states()
    for key_index = 0, num_keys do
        local pressed = get_key_state(key_index)

        if pressed then
            local index = key_index_to_canvas(key_index) + 1

            color_map_afterglow[index] = color_afterglow

            grid[index] = 1.0

            if key_index ~= 0 then
                for i = 1, max_neigh do
                    local neigh_key = key_index_to_canvas(n(
                                                              neighbor_topology[(key_index *
                                                                  max_neigh) + i +
                                                                  table_offset]) +
                                                              1)

                    if neigh_key ~= 0xff then
                        grid[index] = 1.5
                    end
                end
            end
        end
    end
end

function on_render() if effect_ttl > 0 then submit_color_map(color_map) end end

function on_tick(delta)
    ticks = ticks + delta

    if effect_ttl <= 0 then return end

    update_key_states()

    -- compute halo effect
    for index = 1, canvas_size do
        local epsilon = 0.1
        if grid[index] >= epsilon then
            grid[index - 1] = grid[index] - 0.25

            -- compute colors
            local color = hsl_to_color(lerp(0, 360, sin(grid[index])) +
                                           ((ticks % 360) * 3), 1.0, 0.5)

            local r, g, b, a = color_to_rgba(color)
            color_map[index] = rgba_to_color(r, g, b, lerp(0, 255, opacity))
        else
            grid[index - 1] = 0.0
            color_map[index] = 0x000000000
        end

        -- compute effects
        if color_map_effects[index] > 0x00000000 then
            color_map_effects[index] = color_map_effects[index] - 0x0a0a0a0a
            color_map[index] = color_map[index] + color_map_effects[index]
        else
            color_map_effects[index] = 0x00000000
        end

        -- compute afterglow
        if color_map_afterglow[index] > 0x00000000 then
            color_map_afterglow[index] =
                color_map_afterglow[index] - color_step_afterglow
            color_map[key_index] = color_map_afterglow[index]
        else
            color_map_afterglow[index] = 0x00000000
        end

        -- safety net
        if color_map[index] == nil or color_map[index] < 0x00000000 or
            color_map[index] > 0xffffffff then
            color_map[index] = 0x00000000
        end
    end

    effect_ttl = effect_ttl - 1
end
