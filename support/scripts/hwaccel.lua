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
bail_out = true
color_map = {}
offsets = {0, 0, 0}
ticks = 0
shader = nil

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do color_map[i] = 0x00000000 end

    -- verify that the hwaccel interface is available
    local accel_status = hwaccel_status()
    debug("Hwaccel status: " .. stringify(accel_status))

    bail_out = not toboolean(accel_status["acceleration-available"])

    if not bail_out then
        -- hwaccel seems to be available, e.g. Vulkan or OpenGL/OpenCL are available
        shader = load_shader_program(shader_program)
    end
end

function on_mouse_move(rel_x, rel_y, rel_z)
    offsets[1] = offsets[1] - rel_x
    offsets[2] = offsets[2] - rel_y
    offsets[3] = offsets[3] - rel_z
end

function on_render()
    set_uniform(shader, "time", ticks)
    set_uniform(shader, "mouse", {offsets[1], offsets[2]})

    hwaccel_render(shader)
end

function on_tick(delta)
    if bail_out then return end

    ticks = ticks + delta
end
