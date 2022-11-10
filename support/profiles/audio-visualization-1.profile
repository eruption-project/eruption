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


id = '5ac59fa6-e965-45cb-a0da-e87d29713103'
name = "Audio Visualization (1)"
description = "Audio Visualization effect"
active_scripts = [
	'solid.lua',
	'audioviz1.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.'Solid Color']]
type = 'color'
name = 'color_background'
value = 0xff1f1f1f
default = 0xff1f1f1f
