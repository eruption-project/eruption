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


id = '5dc62fa6-e965-45cb-a0da-e87d29713100'
name = 'Preset: Blue and Red'
description = '''Presets for a 'blue and red' color scheme'''
active_scripts = [
	'batique.lua',
	'shockwave.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.Batique]]
type = 'float'
name = 'color_divisor'
value = 2.5
default = 2.5

[[config.Batique]]
type = 'float'
name = 'color_offset'
value = -110.0
default = -110.0

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
