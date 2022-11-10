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
ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end
end

function on_tick(delta)
    ticks = ticks + delta

    if horizontal then
        for y = 0, canvas_height - 1 do
            for x = 0, canvas_width - 1 do
                local i = canvas_width * y + x + 1

                local alpha = (sin((x % canvas_width) / wave_length +
                                       (ticks * direction / speed_divisor)) + 1) *
                                  scale_factor
                local r, g, b = color_to_rgb(color_wave)

                color_map[i] = rgba_to_color(r, g, b,
                                             clamp(alpha * opacity, 0, 255))
            end
        end
    else
        for y = 0, canvas_height - 1 do
            for x = 0, canvas_width - 1 do
                local i = canvas_width * y + x + 1

                local alpha = (sin((y % canvas_height) / wave_length +
                                       (ticks * direction / speed_divisor)) + 1) *
                                  scale_factor
                local r, g, b = color_to_rgb(color_wave)

                color_map[i] = rgba_to_color(r, g, b,
                                             clamp(alpha * opacity, 0, 255))
            end
        end
    end

    submit_color_map(color_map)
end
