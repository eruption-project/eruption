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


id = '5dc62fa8-e969-45cb-a0cd-e87d29713095'
name = 'Blue FX + Color Swirls (Perlin)'
description = 'Blue FX Effect + Color Swirls (Perlin)'
active_scripts = [
    'swirl-perlin.lua',
    'solid.lua',
#   'water.lua',
#   'phonon.lua',
    'halo.lua',
#   'shockwave.lua',
    'impact.lua',
#   'ghost.lua',
#   'raindrops.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_divisor'
value = 1.0
default = 1.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_offset'
value = 0.0
default = 0.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'time_scale'
value = 150.0
default = 150.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'coord_scale'
value = 14.0
default = 14.0

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff2020ff
default = 0xff2020ff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.70
default = 0.70

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false
# default = false

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0x80ffffff
default = 0x80ffffff

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
