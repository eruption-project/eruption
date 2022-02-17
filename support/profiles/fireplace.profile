id = '5dc59fa6-e969-45cb-a0da-e87d28713339'
name = 'Fireplace'
description = 'A fireplace effect'
active_scripts = [
#	'solid.lua',
	'fire.lua',
	'afterhue.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffff2000
default = 0xffff2000

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.45
default = 0.45

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
