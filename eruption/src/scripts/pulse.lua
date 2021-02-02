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
require "easing"

-- global state variables --
color_map = {}
color_map_glow = {}

ticks = 0
saved_time = 0
direction_in = true

local easings = {
	linear = linear,
	inQuad = inQuad,
	outQuad = outQuad,
	inOutQuad = inOutQuad,
	outInQuad = outInQuad,
	inCubic  = inCubic ,
	outCubic = outCubic,
	inOutCubic = inOutCubic,
	outInCubic = outInCubic,
	inQuart = inQuart,
	outQuart = outQuart,
	inOutQuart = inOutQuart,
	outInQuart = outInQuart,
	inQuint = inQuint,
	outQuint = outQuint,
	inOutQuint = inOutQuint,
	outInQuint = outInQuint,
	inSine = inSine,
	outSine = outSine,
	inOutSine = inOutSine,
	outInSine = outInSine,
	inExpo = inExpo,
	outExpo = outExpo,
	inOutExpo = inOutExpo,
	outInExpo = outInExpo,
	inCirc = inCirc,
	outCirc = outCirc,
	inOutCirc = inOutCirc,
	outInCirc = outInCirc,
	inElastic = inElastic,
	outElastic = outElastic,
	inOutElastic = inOutElastic,
	outInElastic = outInElastic,
	inBack = inBack,
	outBack = outBack,
	inOutBack = inOutBack,
	outInBack = outInBack,
	inBounce = inBounce,
	outBounce = outBounce,
	inOutBounce = inOutBounce,
	outInBounce = outInBounce
}

local function get_easing_function(easing_function)
	local result = easings[easing_function]

	if result ~= nil then
		return result
	else
		error("Invalid easing function specified, falling back to linear easing")
		return linear
	end
end

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
		color_map[i] = 0x00000000
	end
end

function on_tick(delta)
	ticks = ticks + delta

	-- calculate pulse effect
	local time = (ticks * pulse_speed) % 256
	if time == 0 then
		direction_in = not direction_in
		saved_time = time
	end

	local val
	if direction_in then
		val = get_easing_function(easing_function_in)(time, saved_time, 255, saved_time + 255)
	else
		val = 255 - get_easing_function(easing_function_out)(time, saved_time, 255, saved_time + 255)
	end

	local r, g, b, alpha = color_to_rgba(color_pulse)

	local color
	if alpha > 0 then
		color = rgba_to_color(r, g, b, clamp(val, 0, 255))
	else
		color = 0x00000000
	end

	-- highlight WASD keys
	color_map[key_to_index['W']] = color
	color_map[key_to_index['A']] = color
	color_map[key_to_index['S']] = color
	color_map[key_to_index['D']] = color

	submit_color_map(color_map)
end
