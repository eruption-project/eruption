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
-- along with Eruption.  If not, see <http:--www.gnu.org/licenses/>.

require "declarations"
require "utilities"
require "debug"

-- global state variables --
ticks = 0
color_map = {}

handle = animal_create(name, speed, len_min, len_max,
					   -- color gradient definitions
					   { 0.4, color1 }, { 0.6, color2 }, { 1.0, color3 }, opacity,
					   -- coefficients of the movement computation
					   { coefficient_1, coefficient_2, coefficient_3,
					     coefficient_4, coefficient_5 })

-- event handler functions --
function on_startup(config)
	for i = 0, canvas_size do
        color_map[i] = 0x00000000
	end
end

function on_tick(delta)
	ticks = ticks + delta

    -- this effect is almost completely implemented in Rust code (see 'plugins/animal.rs')
    if ticks % animation_delay == 0 then
		-- advance the animal's notion of time by 'delta' ticks
		animal_tick(handle, delta)

		-- render the animal
		color_map = animal_render(handle)

		submit_color_map(color_map)
	end
end
