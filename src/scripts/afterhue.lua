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
hue_map = {}
ticks = 0

-- event handler functions --
function on_startup(config)
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        hue_map[i] = 0
				color_map[i] = hsla_to_color(0.0, 0.0, 0.0, 0.0)
    end
end

function on_key_down(key_index)
    hue_map[key_index] = 359
    color_map[key_index] = hsla_to_color(hue_map[key_index], 1.0, 0.5, lerp(0, 255, opacity))
end

function on_tick(delta)
    ticks = ticks + delta + 1

    -- calculate afterhue effect for pressed keys
    if ticks % afterglow_step == 0 then
				local num_keys = get_num_keys()
        for i = 0, num_keys do
						if hue_map[i] > 0 then
							hue_map[i] = hue_map[i] - hue_step_afterglow
							r, g, b, alpha = color_to_rgba(hsl_to_color(hue_map[i], 1.0, 0.5))
							color_map[i] = rgba_to_color(r, g, b, lerp(0, 255, opacity))
						end
        end

	    submit_color_map(color_map)
    end
end
