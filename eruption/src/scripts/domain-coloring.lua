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
ticks = 0
color_map = {}
offsets = {0, 0, 0}

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end
end

function on_mouse_move(rel_x, rel_y, rel_z)
    offsets[1] = offsets[1] - rel_x
    offsets[2] = offsets[2] - rel_y
    offsets[3] = offsets[3] - rel_z
end

function on_render()
    submit_color_map(color_map)
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate domain coloring effect
    if ticks % animation_delay == 0 then
        for i = 0, canvas_size - 1 do
            local x = (i % canvas_width + (offsets[2] / 256)) / coord_scale
            local y = (i / canvas_width + (offsets[1] / 256)) / coord_scale
            local z = (ticks / time_scale + (offsets[3] / 256))

            local m = sqrt(x * x + y * y)
            local t = atan2(y, x)

            z = cos(sqrt(z * z))

            val = m * t * z
            val = lerp(0, 360, val)

            color_map[i + 1] = hsla_to_color(
                                   (val / color_divisor) + color_offset,
                                   color_saturation, color_lightness,
                                   lerp(0, 255, opacity))
        end
    end
end
