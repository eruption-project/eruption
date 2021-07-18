id = '5dc62fa6-e965-45cb-a0da-e87d29713099'
name = 'Preset: Red and Yellow'
description = '''Presets for a 'red and yellow' color scheme'''
active_scripts = [
	'batique.lua',
	'shockwave.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.Batique]]
type = 'float'
name = 'color_divisor'
value = 2.5
default = 2.5

[[config.Batique]]
type = 'float'
name = 'color_offset'
value = 25.0
default = 25.0

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
