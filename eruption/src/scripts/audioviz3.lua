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

ticks = 0
column = 0
power_envelope = 128.0

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
		color_map[i] = 0x00000000
	end
end

function on_tick(delta)
	ticks = ticks + delta

	for i = 0, canvas_size do
		color_map[i] = rgba_to_color(0, 0, 0, lerp(0, 255, opacity))
	end

	local spectrum = get_audio_spectrum()
	local num_buckets = 32
	local num_rows = max_keys_per_col

	for col = 1, num_cols + 1, 1 do
		local bucket = trunc(num_buckets / num_cols * col)
		local val = spectrum[bucket]
		if val == nil then val = 0 end

		local p = trunc(max(num_rows - (val / power_envelope), 0))

		-- debug("Col: " .. col .. " Value: " .. val .. " Envelope: " .. power_envelope ..
		-- 		 " Bucket: " .. bucket .. " p: " .. p)

		for i = num_rows - 1, p, -1 do
			local index = n(rows_topology[col + i * max_keys_per_row]) + 1
			color_map[index] = linear_gradient(color_hot, color_cold, i / num_rows)

			local peak_index = n(rows_topology[col + p * max_keys_per_row]) + 1
			if i ~= p then
				color_map[peak_index] = rgba_to_color(0, 0, 0, lerp(0, 255, opacity))
			end
		end
	end

    submit_color_map(color_map)
end
