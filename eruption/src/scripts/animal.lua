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

handle = animal_create(name, speed, len_min, len_max, max_radius,
-- color gradient definitions
                       {gradient_stop_1, color_1}, {gradient_stop_2, color_2},
                       {gradient_stop_3, color_3}, opacity,
-- coefficients of the movement computation
                       {
    coefficient_1, coefficient_2, coefficient_3, coefficient_4, coefficient_5
})

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = color_background end
end

function on_render()
    -- render the animal
    local animal_map = animal_render(handle)
    for i = 1, num_keys do color_map[i] = animal_map[i] end

    submit_color_map(color_map)
end

function on_tick(delta)
    ticks = ticks + delta

    -- this effect is almost completely implemented in Rust code (see 'plugins/animal.rs')
    if ticks % animation_delay == 0 then
        -- advance the animal's notion of time by 'delta' ticks
        animal_tick(handle, delta)
    end
end
