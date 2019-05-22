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

-- global script configuration --
config["script_name"] = "temperature"
config["script_description"] = "Make the keyboard reflect the system temperature"
config["script_version"] = "0.0.1"
config["script_author"] = "The Eruption development team"
config["min_supported_version"] = "0.0.2"

-- global constants --
color_off = 0x000000000
color_background = 0x00111111
color_cold = rgb_to_color(0, 128, 0)
color_hot = rgb_to_color(128, 0, 0)
color_step = 0x00110000

color_afterglow = rgb_to_color(255, 0, 0)
color_step_afterglow = rgb_to_color(10, 0, 0)
afterglow_step = 2

-- global state variables --
temperature = get_package_temp()
max_temperature = get_package_max_temp()
color_map = {}
color_map_pressed = {}

ticks = 0

-- event handler functions --
function on_startup(config)
    init_state()
    percentage = 0
end

function on_quit(exit_code)
end

function on_key_down(key_index)
    color_map_pressed[key_index] = color_afterglow
end

function on_tick(delta)
    ticks = ticks + delta + 1

    -- update the temperature approximately every 2 seconds
    if ticks % 40 == 0 then
        temperature = get_package_temp()
        trace("Temperature  " .. get_package_temp() .. " / " .. max_temperature)
    end
    
    local num_keys = get_num_keys()

    -- calculate colors
    local percentage = min(temperature / max_temperature * 100, 100)

    for i = 0, num_keys do
        color_map[i] = linear_gradient(color_cold, color_hot, percentage / 100)
    end

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
        color_map_combined[i] = color_map[i] + color_map_pressed[i]

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
    end
end
