id = '5dc59fa6-e965-45cb-a0da-e87d28713395'
name = 'Solid Color + Wave'
description = 'Shows a solid color and a wave effect'
active_scripts = [
	'solid.lua',
	'wave.lua',
	'shockwave.lua',
#	'impact.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffff0000

# [[config.Wave]]
# type = 'bool'
# name = 'horizontal'
# value = true

# [[config.Wave]]
# type = 'int'
# name = 'direction'
# value = -1

# [[config.Wave]]
# type = 'float'
# name = 'scale_factor'
# value = 127.0

# [[config.Wave]]
# type = 'float'
# name = 'speed_divisor'
# value = 25.0

# [[config.Wave]]
# type = 'float'
# name = 'wave_length'
# value = 4.0

[[config.Wave]]
type = 'color'
name = 'color_wave'
value = 0x00ffff00

[[config.Wave]]
type = 'float'
name = 'opacity'
value = 0.5
