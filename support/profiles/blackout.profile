id = '5dc59faf-e965-45cb-a1da-e87d28713395'
name = 'Blackout (all LEDs off)'
description = 'Turn all LEDs off except for overlays and indicators'
active_scripts = [
	'solid.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0x00000000
default = 0x00000000
