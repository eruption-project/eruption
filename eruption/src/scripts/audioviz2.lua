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
require "debug"

-- global state variables --
max_loudness = 64
color_map = {}

ticks = 0
column = 0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = color_background
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- update the state
	local loudness = get_audio_loudness()
	if loudness > max_loudness then
		max_loudness = loudness
	end

	max_loudness = max_loudness * 0.999
	if max_loudness < 8 then
		max_loudness = 8
	end

	-- calculate colors
	local l = loudness / max_loudness
	local color_angle = lerp(0, 360, l)

	if column > max_keys_per_row then
		column = 0
	end

	local color = hsl_to_color(color_angle, 1.0, 0.5)
	for i = 0, num_rows - 1 do
		color_map[column * num_rows + i] = color
	end

	column = column + 1

    submit_color_map(color_map)
end
