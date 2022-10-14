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
-- Copyright (c) 2019-2022, The Eruption Development Team
--
require "declarations"
require "utilities"
require "debug"

-- global state variables --
color_map = {}
ticks = 0
max_effect_ttl = target_fps * 8
effect_ttl = max_effect_ttl
bail_out = true

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    local accel_status = hwaccel_status()
    debug("Hwaccel status: " .. stringify(accel_status))

    bail_out = not toboolean(accel_status["acceleration-available"])
end

function on_key_down(key_index) effect_ttl = max_effect_ttl end

function on_key_up(key_index) effect_ttl = max_effect_ttl end

function on_tick(delta)
    if bail_out then return end

    ticks = ticks + delta

    if effect_ttl <= 0 then return end

    effect_ttl = effect_ttl - 1

    hwaccel_tick(delta)

    color_map = color_map_from_render_surface()
    submit_color_map(color_map)
end
