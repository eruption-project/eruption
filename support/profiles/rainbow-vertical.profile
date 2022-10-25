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


id = '5cd59fa6-e965-45cb-a0da-e87d29713123'
name = 'Rainbow Animation (vertical)'
description = 'Display an animated rainbow'
active_scripts = [
    'stock-gradient.lua',
#   'halo.lua',
    'shockwave.lua',
#   'impact.lua',
#   'water.lua',
#   'raindrops.lua',
#   'sysmon.lua',
    'batique.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Stock Gradient"]]
type = 'string'
name = 'stock_gradient'
value = "sinebow-smooth"
default = "sinebow-smooth"

[[config."Stock Gradient"]]
type = 'bool'
name = 'horizontal'
value = false
default = false

[[config."Stock Gradient"]]
type = 'int'
name = 'direction'
value = 1
default = 1

[[config."Stock Gradient"]]
type = 'float'
name = 'wave_length'
value = 5.0
default = 5.0

[[config."Stock Gradient"]]
type = 'float'
name = 'speed_divisor'
value = 64.0
default = 64.0

[[config.Shockwave]]
type = 'color'
name = 'color_shockwave'
value = 0xffff5f00
default = 0xffff5f00

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

# mouse support
[[config.Batique]]
type = 'int'
name = 'zone_start'
value = 144
default = 144

[[config.Batique]]
type = 'int'
name = 'zone_end'
value = 180
default = 180

[[config.Batique]]
type = 'float'
name = 'coord_scale'
value = 180
default = 180

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
value = 1.0
default = 1.0
