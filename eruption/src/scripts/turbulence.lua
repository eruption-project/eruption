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
require "debug"

-- global state variables --
ticks = 0
color_map = {}

function on_tick(delta)
    ticks = ticks + delta

    -- calculate turbulence noise effect
    if ticks % animation_delay == 0 then
        for i = 0, canvas_size - 1 do
            local x = i % canvas_width
            local y = i / canvas_width

            local val = turbulence_noise_3d(x / coord_scale, y / coord_scale,
                                            ticks / time_scale)
            val = lerp(0, 360, val)

            color_map[i + 1] = hsla_to_color(
                                   (val / color_divisor) + color_offset,
                                   color_saturation, color_lightness,
                                   lerp(0, 255, opacity))
        end

        submit_color_map(color_map)
    end
end
