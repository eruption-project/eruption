#    SPDX-License-Identifier: GPL-3.0-or-later
#
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


id = '5dc59fa6-e965-45cb-a0da-e87d28713295'
name = 'Rainbow + Wave'
description = 'Shows a rainbow + a wave effect'
active_scripts = [
	'rainbow.lua',
	'wave.lua',
	'shockwave.lua',
#	'impact.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

# [[config.Wave]]
# type = 'bool'
# name = 'horizontal'
# value = true
# default = true

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

# [[config.Wave]]
# type = 'float'
# name = 'speed_divisor'
# value = 25.0
# default = 25.0

# [[config.Wave]]
# type = 'float'
# name = 'wave_length'
# value = 4.0
# default = 4.0

# [[config.Wave]]
# type = 'color'
# name = 'color_wave'
# value = 0x00000000
# default = 0x00000000

# [[config.Wave]]
# type = 'float'
# name = 'opacity'
# value = 1.0
# default = 1.0

# dim a specific zone, e.g. if the mouse LEDs are too bright
[[config."Dim Zone"]]
type = 'int'
name = 'zone_start'
value = 144
default = 144

[[config."Dim Zone"]]
type = 'int'
name = 'zone_end'
value = 180
default = 180

[[config."Dim Zone"]]
type = 'float'
name = 'opacity'
value = 0.95
default = 0.95
