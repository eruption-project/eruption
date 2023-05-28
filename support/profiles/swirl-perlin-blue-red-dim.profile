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

id = '5dc62fa6-e966-45cb-a0da-e87d29713118'
name = 'Color Swirls (Perlin): Blue and Red (dimmed)'
description = 'Color Swirl effect with high contrast'
active_scripts = [
    'solid.lua',
    'swirl-perlin.lua',
    'halo.lua',
    'shockwave.lua',
#   'impact.lua',
#   'water.lua',
#   'raindrops.lua',
#   'sysmon.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff000000
default = 0xff000000

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_divisor'
value = 4.0
default = 4.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_offset'
value = -110.0
default = -110.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'time_scale'
value = 250.0
default = 250.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'coord_scale'
value = 15.0
default = 15.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'opacity'
value = 0.15
default = 0.15

[[config.Shockwave]]
type = 'color'
name = 'color_step_shockwave'
value = 0x05010000
default = 0x05010000

[[config.Shockwave]]
type = 'bool'
name = 'mouse_events'
value = true
default = true

[[config.Shockwave]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0xa0ff0000
default = 0xa0ff0000

[[config.Shockwave]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0xd0ff0000
default = 0xd0ff0000

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.75
default = 0.75

[[config."System Monitor"]]
type = 'color'
name = 'color_cold'
value = 0x0000ff00
default = 0x0000ff00

[[config."System Monitor"]]
type = 'color'
name = 'color_hot'
value = 0xffff0000
default = 0xffff0000
