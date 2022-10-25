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

matrix = require "matrix"

-- global state variables --
ticks = 0
color_map = {}

grad = gradient_from_name(stock_gradient)

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0xffffffff end
end

function on_tick(delta)
    ticks = ticks + delta

    local theta = 360 * sin(ticks / 5000)

    local tx = canvas_width / 2
    local ty = canvas_height / 2

    local scale_x = 1.0
    local scale_y = 1.0

    local trans_mat1 = matrix{{1, 0, tx}, {0, 1, ty}, {0, 0, 1}}:transpose()
    local trans_mat2 = matrix{{1, 0, -tx}, {0, 1, -ty}, {0, 0, 1}}:transpose()
    local scaling_mat = matrix{{scale_x, 0, 0}, {0, scale_y, 0}, {0, 0, 1}}:transpose()
    local rot_mat = matrix{{cos(theta), -sin(theta), 0}, {sin(theta), cos(theta), 0}, {0, 0, 1}}:transpose()

    -- local affine_mat = rot_mat * scaling_mat * trans_mat
    -- local affine_mat = trans_mat * scaling_mat * rot_mat  
    -- local affine_mat = scaling_mat * trans_mat2 * rot_mat * trans_mat1
    -- local affine_mat = scaling_mat * trans_mat1 * rot_mat * trans_mat2 
    local affine_mat = rot_mat

    for i = 0, canvas_size - 1 do
	local x = i % canvas_width
	local y = i / canvas_width

	local val = sin((ticks * direction * wave_length) / speed_divisor)
	local grad_pos = range(-1.0, 1.0, 0.0, 1.0, val)
	local color = gradient_color_at(grad, grad_pos)

	-- finer grained color control
	-- local h,s,l = color_to_hsl(color)
	-- h = range(-120.0, 120.0, 0.0, 360.0, h)
	-- color = hsla_to_color(h, (s * color_saturation) * 2, (l * color_lightness * 2), 255)

	local vec = matrix{x, y, 1}

	vec = trans_mat1 * vec
	vec = affine_mat * vec 
	vec = trans_mat2 * vec

	warn("" .. stringify(vec))

	x = trunc(vec[1][1])
	y = trunc(vec[2][1])

	local index = n(rows_topology[x + (y * max_keys_per_row)])
	color_map[index + 1] = color
    end

    submit_color_map(color_map)
end
