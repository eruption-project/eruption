#    This file is part of Eruption.
#
#    Eruption is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    Eruption is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
#
#    Copyright (c) 2019-2022, The Eruption Development Team


id = '5dc59fa6-e965-45cb-a0da-e87d28713337'
name = 'Matrix'
description = 'Shows a matrix like effect'
active_scripts = [
	'solid.lua',
	'ghost.lua',
	'wave.lua',
	'afterhue.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff10ff10
default = 0xff10ff10

[[config.Ghost]]
type = 'int'
name = 'ghost_backoff_secs'
value = 1
default = 1

[[config.Ghost]]
type = 'float'
name = 'ghost_intensity'
value = 2.5
default = 2.5

[[config.Ghost]]
type = 'float'
name = 'afterglow_step'
value = 1.0
default = 1.0

[[config.Ghost]]
type = 'color'
name = 'color_afterglow'
value = 0x20ffffff
default = 0x20ffffff

[[config.Ghost]]
type = 'color'
name = 'color_step_afterglow'
value = 0x0a0a0a0a
default = 0x0a0a0a0a

[[config.Ghost]]
type = 'color'
name = 'color_shockwave'
value = 0xa0ff0000
default = 0xa0ff0000

[[config.Ghost]]
type = 'color'
name = 'color_step_shockwave'
value = 0x05050505
default = 0x05050505

[[config.Ghost]]
type = 'int'
name = 'shockwave_divisor'
value = 12
default = 12

[[config.Wave]]
type = 'bool'
name = 'horizontal'
value = false
default = false

# [[config.Wave]]
# type = 'int'
# name = 'direction'
# value = -1
# default = -1

# [[config.Wave]]
# type = 'float'
# name = 'scale_factor'
# value = 127.0
# default = 127.0

[[config.Wave]]
type = 'float'
name = 'speed_divisor'
value = 20.0
default = 20.0

[[config.Wave]]
type = 'float'
name = 'wave_length'
value = 1.0
default = 1.0

# [[config.Wave]]
# type = 'color'
# name = 'color_wave'
# value = 0x00000000
# default = 0x00000000

[[config.Wave]]
type = 'float'
name = 'opacity'
value = 1.0
default = 1.0
