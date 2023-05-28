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

id = '59c62fa6-d865-55cb-a0ef-e87d29713119'
name = 'Shader Demo'
description = 'Shader Demo (Hardware accelerated)'
active_scripts = [
    'hwaccel.lua',
#   'impact.lua',
#   'raindrops.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Hardware Acceleration"]]
type = 'string'
name = 'shader_program'
value = 'shaders/mandelbrot.comp.glsl'
default = 'shaders/mandelbrot.comp.glsl'
