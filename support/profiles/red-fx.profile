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

id = '5dc62fa6-e965-45cb-a0da-e87d29713059'
name = 'Red FX'
description = 'Red FX'
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
value = 0xffff000c
default = 0xffff000c

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.45
default = 0.45

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false
# default = false

[[config.Halo]]
type = 'color'
name = 'color_afterglow'
value = 0xffffaf00
default = 0xffffaf00

[[config.Halo]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0x1affffff
default = 0x1affffff

[[config.Halo]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0x1affffff
default = 0x1affffff

# [[config.Halo]]
# type = 'float'
# name = 'opacity'
# value = 0.25
# default = 0.25

[[config.Shockwave]]
type = 'color'
name = 'color_shockwave'
value = 0xffff5f00
default = 0xffff5f00

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0x80ffff00
default = 0x80ffff00

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5
default = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x000f0000
default = 0x000f0000
