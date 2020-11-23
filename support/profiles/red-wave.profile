id = '5dc59fa6-a965-45cb-a0cd-e87d28713323'
name = 'Red Wave'
description = 'Red wave effect'
active_scripts = [
	'solid.lua',
	'wave.lua',
	'halo.lua',
#	'shockwave.lua',
#	'impact.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffff0509

# [[config."Solid Color"]]
# type = 'float'
# name = 'opacity'
# value = 1.0

[[config.Wave]]
type = 'bool'
name = 'horizontal'
value = false

# [[config.Wave]]
# type = 'int'
# name = 'direction'
# value = -1

# [[config.Wave]]
# type = 'float'
# name = 'scale_factor'
# value = 127.0

[[config.Wave]]
type = 'float'
name = 'speed_divisor'
value = 32.0

[[config.Wave]]
type = 'float'
name = 'wave_length'
value = 1.5

[[config.Wave]]
type = 'color'
name = 'color_wave'
value = 0xffffff00

[[config.Wave]]
type = 'float'
name = 'opacity'
value = 0.25

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false

[[config.Halo]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0x2fffffff

[[config.Halo]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0x4fffffff

[[config.Shockwave]]
type = 'color'
name = 'color_shockwave'
value = 0xffff2f2f

[[config.Shockwave]]
type = 'color'
name = 'color_afterglow'
value = 0xffffafaf

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0xffe59400
