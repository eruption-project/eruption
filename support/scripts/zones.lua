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
require "debug"

-- global state variables --
color_map = {}
max_effect_ttl = target_fps * 3
effect_ttl = max_effect_ttl

function realize()
    -- clear the color map
    for i = 0, canvas_size do color_map[i] = 0x00000000 end

    -- render each zone in a different color
    for z = 0, #zones do
        local hue_increment = 360 / (#zones + 1)

        for y = zones[z].y, zones[z].y2 - 1 do
            for x = zones[z].x, zones[z].x2 - 1 do
                local index = (y * canvas_width) + x + 1
                color_map[index] = hsla_to_color(hue_increment * (z + 1), 1.0,
                                                 0.5, lerp(0, 255, opacity))
            end
        end
    end

    submit_color_map(color_map)
end

function on_startup(config) realize() end

function on_apply_parameter(parameters) realize() end

function on_render() if effect_ttl > 0 then submit_color_map(color_map) end end

function on_tick(delta)
    if effect_ttl <= 0 then return end

    effect_ttl = effect_ttl - 1

    realize()
end
