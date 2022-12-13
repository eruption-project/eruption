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

id = '5dc24fa6-e965-45cb-a0da-e87d28713095'
name = 'Psychedelic Twinkle'
description = 'The psychedelic twinkle profile'
active_scripts = [
	'psychedelic.lua',
	'shockwave.lua',
#	'impact.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Psychedelic"]]
type = 'float'
name = 'color_boost'
value = 25
default = 25

[[config."Psychedelic"]]
type = 'float'
name = 'saturation_boost'
value = 2
default = 2
