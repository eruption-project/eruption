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
#    Copyright (c) 2019-2023, The Eruption Development Team


id = '5cd23fa6-e965-45cb-a0cd-e87d28713091'
name = 'Animal: Blobby + Wave'
description = 'Simulate a lifeform with organic movements'
active_scripts = [
	'solid.lua',
    'wave.lua',
	'animal.lua',
    'afterhue.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.15
default = 0.15

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
value = 32.0
default = 32.0

[[config.Wave]]
type = 'float'
name = 'wave_length'
value = 1.5
default = 1.5

[[config.Wave]]
type = 'color'
name = 'color_wave'
value = 0xffffff00
default = 0xffffff00

[[config.Wave]]
type = 'float'
name = 'opacity'
value = 0.05
default = 0.05

[[config.Animal]]
type = 'string'
name = 'name'
value = 'Blobby'
default = 'Blobby'

[[config.Animal]]
type = 'float'
name = 'opacity'
value = 0.65
default = 0.65

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
