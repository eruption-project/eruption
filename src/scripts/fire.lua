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

-- global state variables --
color_map = {}
color_map_pressed = {}
color_map_fire = {}

-- compute fire with a supersampling factor of 8
fire_grid_rows = 6 * 8
fire_grid_cols = 22 * 8

fire_grid = {}
color_palette = {}

ticks = 0

-- event handler functions --
function on_startup(config)
    init_state()
end

function on_quit(exit_code)
    init_state()
    set_color_map(color_map)
end

function on_key_down(key_index)
    color_map_pressed[key_index] = color_afterglow
end

function compute_fire(ticks)
    local num_keys = get_num_keys()

    if ticks % fire_speed == 0 then
        -- randomize bottom row
        for x = 0, fire_grid_cols - 1 do
            fire_grid[fire_grid_rows * x + fire_grid_cols] = rand(55, 255)
        end

        -- compute fire from top to bottom
        for y = 0, fire_grid_rows - 2 do
            for x = 0, fire_grid_cols - 1 do
                fire_grid[y * fire_grid_cols + x] = 
                                ((fire_grid[((y + 1) % fire_grid_rows) * fire_grid_cols + ((x - 1 + fire_grid_cols) % fire_grid_cols)]
                                + fire_grid[((y + 1) % fire_grid_rows) * fire_grid_cols + ((x) % fire_grid_cols)]
                                + fire_grid[((y + 1) % fire_grid_rows) * fire_grid_cols + ((x + 1) % fire_grid_cols)]
                                + fire_grid[((y + 2) % fire_grid_rows) * fire_grid_cols + ((x) % fire_grid_cols)])
                                * 32) / 129;
            end
        end

        for y = 1, fire_grid_rows - 2 do
            for x = 1, fire_grid_cols - 2 do
                -- compute average (downsample)
                local sum = fire_grid[(y-1) * fire_grid_cols + (x-1)] +
                            fire_grid[(y-1) * fire_grid_cols + (x)]   +
                            fire_grid[(y-1) * fire_grid_cols + (x+1)] +
                            fire_grid[(y-1) * fire_grid_cols + (x)]   +
                            fire_grid[(y+1) * fire_grid_cols + (x)]   +
                            fire_grid[(y+1) * fire_grid_cols + (x-1)] +
                            fire_grid[(y+1) * fire_grid_cols + (x)]   +
                            fire_grid[(y+1) * fire_grid_cols + (x+1)]
                
                local avg = sum / 8
                
                local idx = (x / 8) * fire_grid_rows + (y / 8)
                color_map_fire[idx] = color_palette[trunc(avg)]

                -- should not happen, but be safe
                if color_map_fire[idx] == nil then
                    color_map_fire[idx] = color_palette[0]
                end
            end
        end
    end
end

function on_tick(delta)
    ticks = ticks + delta + 1

    local num_keys = get_num_keys()

    -- calculate fire effect
    compute_fire(ticks)

    -- calculate afterglow effect for pressed keys
    if ticks % afterglow_step == 0 then
        for i = 0, num_keys do        
            if color_map_pressed[i] >= 0x00000000 then
                color_map_pressed[i] = color_map_pressed[i] - color_step_afterglow

                if color_map_pressed[i] >= 0x00ffffff then
                    color_map_pressed[i] = 0x00ffffff
                elseif color_map_pressed[i] <= 0x00000000 then
                    color_map_pressed[i] = 0x00000000
                end
            end
        end
    end

    -- now combine all the color maps to a final map
    local color_map_combined = {}
    for i = 0, num_keys do
        color_map_combined[i] = color_map[i] + color_map_fire[i] + color_map_pressed[i]

        -- let the afterglow effect override all other effects
        if color_map_pressed[i] > 0x00000000 then
            color_map_combined[i] = color_map_pressed[i]
        end

        if color_map_combined[i] >= 0x00ffffff then
            color_map_combined[i] = 0x00ffffff
        elseif color_map_combined[i] <= 0x00000000 then
            color_map_combined[i] = 0x00000000
        end
    end

    set_color_map(color_map_combined)
end

-- init global state
function init_state()
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        color_map[i] = color_background
        color_map_pressed[i] = color_off
        color_map_fire[i] = color_off
    end

    -- initialize fire grid
    for y = 0, fire_grid_rows do
        for x = 0, fire_grid_cols do
            fire_grid[x * fire_grid_rows + y] = 0
        end
    end

     -- initialize palette
    for i = 0, 255 do
        color_palette[i] = hsl_to_color(i / 3, 1.0, min(0.5, ((i * 1.45) / 256)))
    end
end
