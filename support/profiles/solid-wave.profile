id = '5dc59fa6-e965-45cb-a0da-e87d28713395'
name = 'Solid Color + Wave'
description = 'Shows a solid color and a wave effect'
active_scripts = [
	'solid.lua',
	'wave.lua',
	'shockwave.lua',
#	'impact.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffff0000
default = 0xffff0000

# [[config.Wave]]
# type = 'bool'
# name = 'horizontal'
# value = true
# default = true

# [[config.Wave]]
# type = 'int'
# name = 'direction'
# value = -1
# default = -1

# [[config.Wave]]
# type = 'float'
# name = 'scale_factor'
# value = 127.0
# default = 127.0

# [[config.Wave]]
# type = 'float'
# name = 'speed_divisor'
# value = 25.0
# default = 25.0

# [[config.Wave]]
# type = 'float'
# name = 'wave_length'
# value = 4.0
# default = 4.0

[[config.Wave]]
type = 'color'
name = 'color_wave'
value = 0x00ffff00
default = 0x00ffff00

[[config.Wave]]
type = 'float'
name = 'opacity'
value = 0.5
default = 0.5

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
