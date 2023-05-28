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
power_envelope = 16.0

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do color_map[i] = 0x00000000 end
end

function on_render() submit_color_map(color_map) end

function on_tick(delta)
    ticks = ticks + delta

    for i = 1, canvas_size do
        color_map[i] = rgba_to_color(0, 0, 0, lerp(0, 255, opacity))
    end

    local spectrum = get_audio_spectrum()
    local num_buckets = canvas_width

    for col = 1, canvas_width do
        local bucket = trunc(num_buckets / canvas_width * col)
        local val = n(spectrum[bucket])

        local p = trunc(max(canvas_height - (val / power_envelope), 0))

        -- debug("Col: " .. col .. " Value: " .. val .. " Envelope: " .. power_envelope ..
        -- 		 " Bucket: " .. bucket .. " p: " .. p)

        for i = canvas_height - 1, p, -1 do
            local index = col + i * canvas_width
            color_map[index] = linear_gradient(color_hot, color_cold,
                                               i / canvas_height)

            local peak_index = col + p * canvas_width
            if i ~= p then
                color_map[peak_index] = rgba_to_color(0, 0, 0,
                                                      lerp(0, 255, opacity))
            end
        end
    end
end
