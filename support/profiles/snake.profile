id = '5dc24fa6-e965-45cb-a0cd-e87d28713091'
name = 'Snake'
description = 'The snake profile'
active_scripts = [
    'solid.lua',
    'afterhue.lua',
	'snake.lua',
#   'raindrops.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffffffff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.05

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5
