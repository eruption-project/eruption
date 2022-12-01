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
ticks = 0

grad = gradient_from_name(stock_gradient)

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate the gradient effect
    if horizontal then
        for i = num_cols, 0, -1 do
            for j = 1, max_keys_per_col do
                local val = sin(i / wave_length +
                                    (ticks * direction / speed_divisor)) *
                                scale_factor
                local grad_pos = range(-1.0, 1.0, 0.0, 1.0, val)
                local color = gradient_color_at(grad, grad_pos)

                local index = n(cols_topology[j + (i * max_keys_per_col)]) + 1
                color_map[index] = color
            end
        end
    else
        for i = num_rows, 0, -1 do
            for j = 1, max_keys_per_row do
                local val = sin(i / wave_length +
                                    (ticks * direction / speed_divisor)) *
                                scale_factor
                local grad_pos = range(-1.0, 1.0, 0.0, 1.0, val)
                local color = gradient_color_at(grad, grad_pos)

                local index = n(rows_topology[j + (i * max_keys_per_row)]) + 1
                color_map[index] = color
            end
        end
    end

    submit_color_map(color_map)
end
