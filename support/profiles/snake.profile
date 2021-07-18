id = '5dc24fa6-e965-45cb-a0cd-e87d28713091'
name = 'Snake'
description = 'The snake profile'
active_scripts = [
    'solid.lua',
    'afterhue.lua',
	'snake.lua',
#   'raindrops.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffffffff
default = 0xffffffff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.05
default = 0.05

[[config.Raindrops]]
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
