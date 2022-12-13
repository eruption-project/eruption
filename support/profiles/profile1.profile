#  SPDX-License-Identifier: GPL-3.0-or-later
#
#  This file is part of Eruption.
#
#  Eruption is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  Eruption is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
#
#  Copyright (c) 2019-2023, The Eruption Development Team

id = '5dc62fa6-e965-45cb-a0da-e87d29713095'
name = 'Blue FX'
description = 'Default Profile #1'
active_scripts = [
    'solid.lua',
#   'water.lua',
#   'phonon.lua',
    'halo.lua',
    'shockwave.lua',
    'impact.lua',
#   'ghost.lua',
#   'raindrops.lua',
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
value = 0.45
default = 0.45

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false

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
