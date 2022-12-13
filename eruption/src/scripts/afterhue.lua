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
hue_map = {}

ticks = 0
max_effect_ttl = target_fps * 8
effect_ttl = max_effect_ttl

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do
        hue_map[i] = 0
        color_map[i] = hsla_to_color(0.0, 0.0, 0.0, 0.0)
    end
end

function on_key_down(key_index) effect_ttl = max_effect_ttl end

function on_key_up(key_index) effect_ttl = max_effect_ttl end

local function update_key_states()
    for key_index = 1, num_keys do
        local pressed = get_key_state(key_index)

        if pressed then
            local index = key_index_to_canvas(key_index) + 1

            hue_map[index] = 359
            color_map[index] = hsla_to_color(hue_map[index], 1.0, 0.5,
                                             lerp(0, 255, opacity))
        end
    end
end

function on_render()
    if effect_ttl > 0 then
        submit_color_map(color_map)
    end
end

function on_tick(delta)
    ticks = ticks + delta

    if effect_ttl <= 0 then return end

    update_key_states()

    -- calculate afterhue effect for pressed keys
    if ticks % afterglow_step == 0 then
        for i = 1, canvas_size do
            if hue_map[i] > 0 then
                hue_map[i] = hue_map[i] - hue_step_afterglow

                if hue_map[i] >= hue_step_afterglow then
                    r, g, b, alpha = color_to_rgba(
                                         hsl_to_color(hue_map[i], 1.0, 0.5))
                    color_map[i] = rgba_to_color(r, g, b, lerp(0, 255, opacity))
                else
                    color_map[i] = 0x00000000
                end
            end
        end

        effect_ttl = effect_ttl - 1
    end
end
