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

    -- animate gradient
    if ticks % gradient_step == 0 then
        for i = 0, canvas_height - 1 do
            local color = hsla_to_color((i + ticks * gradient_speed) + (10 * i),
                                        1.0, 0.5, lerp(0, 255, opacity))

            for j = 0, canvas_width - 1 do
                local index = j + 1 + (i * canvas_width)
                color_map[index] = color
            end
        end

        submit_color_map(color_map)
    end
end
