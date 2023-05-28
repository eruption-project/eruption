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
require "debug"

-- global state variables --
ticks = 0
offsets = {0, 0, 0}

canvas = get_canvas()
layer1 = create_new_canvas()

-- event handler functions --
function on_startup(config)
    set_source_color(layer1, 0xff0000ff)
    fill_rectangle(layer1, 0, 0, canvas_width, canvas_height)
end

function on_mouse_move(rel_x, rel_y, rel_z)
    offsets[1] = offsets[1] - rel_x
    offsets[2] = offsets[2] - rel_y
    offsets[3] = offsets[3] - rel_z
end

function on_render()
    -- local canvas = get_canvas()

    -- -- testing simplex noise instead of the umbrella effect for now
    -- draw_simplex_noise(canvas, 0, 0, canvas_width, canvas_height,
    --                    offsets[1] / 20, offsets[2] / 20, ticks, 0.25)

    -- realize_canvas(canvas)

    -- testing simplex noise instead of the umbrella effect for now
    draw_simplex_noise(canvas, 0, 0, canvas_width, canvas_height,
                       offsets[1] / 20, offsets[2] / 20, ticks, 0.25)

    alpha_blend(layer1, canvas, 0.5)

    realize_canvas(canvas)
end

function on_tick(delta) ticks = ticks + delta end
