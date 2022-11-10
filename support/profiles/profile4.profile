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


id = '5dc62fa6-e965-45cb-a0da-e87d29713098'
name = "Linear Gradient"
description = "Default Profile #4"
active_scripts = [
	'linear-gradient.lua',
#	'impact.lua',
	'afterhue.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.Gradient]]
type = 'color'
name = 'color_start'
value = 0xffff0000
default = 0xffff0000

[[config.Gradient]]
type = 'color'
name = 'color_end'
value = 0xff0000ff
default = 0xff0000ff

[[config.Gradient]]
type = 'int'
name = 'color_divisor'
value = 5
default = 5

[[config.Gradient]]
type = 'bool'
name = 'animate_gradient'
value = true
default = true
