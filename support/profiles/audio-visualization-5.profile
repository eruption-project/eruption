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


id = '5ac59fa6-e965-45cb-a0da-e87d29713107'
name = "Audio Visualization (5)"
description = "Audio Visualization effect"
active_scripts = [
	'domain-coloring.lua',
	'audioviz5.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Audio Visualizer #5"]]
type = 'float'
name = 'opacity'
value = 0.95
default = 0.95

[[config."Audio Visualizer #5"]]
type = 'color'
name = 'color_step'
value = 0xffffffff
default = 0xffffffff

[[config."Domain Coloring"]]
type = 'float'
name = 'color_divisor'
value = 1.0
default = 1.0

[[config."Domain Coloring"]]
type = 'float'
name = 'color_offset'
value = 0.0
default = 0.0

[[config."Domain Coloring"]]
type = 'float'
name = 'time_scale'
value = 50.0
default = 50.0

[[config."Domain Coloring"]]
type = 'float'
name = 'coord_scale'
value = 30.0
default = 30.0

[[config."Domain Coloring"]]
type = 'float'
name = 'opacity'
value = 1.0
default = 1.0

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
