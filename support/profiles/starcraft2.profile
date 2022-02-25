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


id = '5dc23fa9-e965-45cb-a0cd-e87d28713091'
name = 'Gaming: StarCraft 2'
description = 'The Star Craft 2 gaming profile'
active_scripts = [
    'solid.lua',
#   'impact.lua',
    'shockwave.lua',
#   'water.lua',
#   'raindrops.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff2020ff
default = 0xff2020ff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.35
default = 0.35

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5
default = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x0004040f
default = 0x0004040f

[[config.Macros]]
type = 'string'
name = 'requires'
value = 'macros/starcraft2'
default = 'macros/starcraft2'

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
