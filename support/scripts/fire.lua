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

fire_grid_cols = canvas_width
fire_grid_rows = canvas_height

fire_grid = {}
color_palette = {}

ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do color_map[i] = 0x00000000 end

    -- initialize fire grid
    for y = 0, fire_grid_rows do
        for x = 0, fire_grid_cols do
            fire_grid[x + y * fire_grid_cols + 1] = 1
        end
    end

    -- initialize palette
    for i = 1, 256 do
        color_palette[i] = hsla_to_color(i + 300, 1.0,
                                         min(1.0, ((i * 1.45) / 256)),
                                         min(255, i))
    end
end

function on_render() submit_color_map(color_map) end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate fire effect
    if ticks % fire_speed == 0 then
        -- randomize bottom row(s)
        for y = fire_grid_rows - 2, fire_grid_rows do
            for x = 0, fire_grid_cols do
                fire_grid[x + (y * (fire_grid_cols))] = rand(1, 255)
            end
        end

        -- compute fire effect, from top to bottom
        for y = 0, fire_grid_rows - 1 do
            for x = 0, fire_grid_cols do
                fire_grid[x + y * fire_grid_cols] = ((n(
                                                        fire_grid[((y + 1) %
                                                            fire_grid_rows) *
                                                            fire_grid_cols +
                                                            ((x - 1 +
                                                                fire_grid_cols) %
                                                                fire_grid_cols)]) +
                                                        n(
                                                            fire_grid[((y + 1) %
                                                                fire_grid_rows) *
                                                                fire_grid_cols +
                                                                ((x) %
                                                                    fire_grid_cols)]) +
                                                        n(
                                                            fire_grid[((y + 1) %
                                                                fire_grid_rows) *
                                                                fire_grid_cols +
                                                                ((x + 1) %
                                                                    fire_grid_cols)]) +
                                                        n(
                                                            fire_grid[((y + 2) %
                                                                fire_grid_rows) *
                                                                fire_grid_cols +
                                                                ((x) %
                                                                    fire_grid_cols)])) *
                                                        31) / 128;
            end
        end

        for y = 0, fire_grid_rows - 1 do
            for x = 0, fire_grid_cols do
                -- compute average
                local sum = n(fire_grid[(y - 1) * fire_grid_cols + (x - 1)]) +
                                n(fire_grid[(y - 1) * fire_grid_cols + (x)]) +
                                n(fire_grid[(y - 1) * fire_grid_cols + (x + 1)]) +
                                n(fire_grid[(y - 1) * fire_grid_cols + (x)]) +
                                n(fire_grid[(y + 1) * fire_grid_cols + (x)]) +
                                n(fire_grid[(y + 1) * fire_grid_cols + (x - 1)]) +
                                n(fire_grid[(y + 1) * fire_grid_cols + (x)]) +
                                n(fire_grid[(y + 1) * fire_grid_cols + (x + 1)])

                local avg = trunc(sum / 8)

                color_map[x + (y * fire_grid_cols)] = n(
                                                          color_palette[clamp(
                                                              avg, 1, 255)])
            end
        end
    end
end