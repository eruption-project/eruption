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

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = color_background
    end
end

function on_tick(delta)
    ticks = ticks + delta

    -- update the state
	loudness = get_audio_loudness()
	if loudness > max_loudness then
		max_loudness = loudness
	end

	max_loudness = max_loudness * 0.999
	if max_loudness < 8 then
		max_loudness = 8
	end

	-- debug("AudioViz1: Loudness  " .. loudness .. " / " .. max_loudness)

    -- calculate colors
    local percentage = min(loudness / max_loudness * 100, 100)

	color = linear_gradient(color_silence, color_loud, percentage / 100)
    for i = 0, canvas_size do
		color_map[i] = color
	end

    submit_color_map(color_map)
end
