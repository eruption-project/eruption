-- This file is part of Eruption.

-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.

-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.

-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}

-- compute fire with a supersampling factor of 2
fire_grid_rows = num_rows * 2
fire_grid_cols = num_cols * 2

fire_grid = {}
color_palette = {}

ticks = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end

    -- initialize fire grid
    for y = 0, fire_grid_rows do
        for x = 0, fire_grid_cols do
            fire_grid[x * fire_grid_rows + y] = 0
        end
    end

     -- initialize palette
    for i = 0, 255 do
        color_palette[i] = hsla_to_color(i / 3, 1.0, min(0.5, ((i * 1.45) / 256)),
                                         lerp(0, 255, opacity))
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- calculate fire effect
    if ticks % fire_speed == 0 then
        -- randomize bottom row
        for x = 0, fire_grid_cols do
            fire_grid[fire_grid_rows * x + fire_grid_cols] = rand(35, 255)
        end

        -- compute fire effect, from top to bottom
        for y = 0, fire_grid_rows - 2 do
            for x = 0, fire_grid_cols - 1 do
                fire_grid[y * fire_grid_cols + x] =
                                ((fire_grid[((y + 1) % fire_grid_rows) *
                                fire_grid_cols + ((x - 1 + fire_grid_cols) % fire_grid_cols)]

                                + fire_grid[((y + 1) % fire_grid_rows) *
                                    fire_grid_cols + ((x) % fire_grid_cols)]

                                + fire_grid[((y + 1) % fire_grid_rows) *
                                    fire_grid_cols + ((x + 1) % fire_grid_cols)]

                                + fire_grid[((y + 2) % fire_grid_rows) *
                                    fire_grid_cols + ((x) % fire_grid_cols)])
                                * 32) / 129;
            end
        end

        for y = 1, fire_grid_rows - 2 do
            for x = 1, fire_grid_cols - 2 do
                -- compute average (downsample)
                local sum = fire_grid[(y - 1) * fire_grid_cols + (x - 1)] +
                            fire_grid[(y - 1) * fire_grid_cols + (x)]     +
                            fire_grid[(y - 1) * fire_grid_cols + (x + 1)] +
                            fire_grid[(y - 1) * fire_grid_cols + (x)]     +
                            fire_grid[(y + 1) * fire_grid_cols + (x)]     +
                            fire_grid[(y + 1) * fire_grid_cols + (x - 1)] +
                            fire_grid[(y + 1) * fire_grid_cols + (x)]     +
                            fire_grid[(y + 1) * fire_grid_cols + (x + 1)]

                local avg = trunc(sum / 8)
                local idx = n(rows_topology[trunc(x + (y * max_keys_per_row))])

                if idx ~= nil then
                    color_map[idx] = color_palette[clamp(avg, 0, 255)]
                end
            end
        end

        submit_color_map(color_map)
    end
end
