id = '5dc59fa6-e965-25cb-a0da-e87d28713094'
name = 'Batique (Mouse only)'
description = 'The batique profile (Mouse only)'
active_scripts = [
	'batique.lua',
#   'dim-zone.lua',
	'macros.lua',
#	'stats.lua',
]

[[config.Batique]]
type = 'int'
name = 'zone_start'
value = 144
default = 144

[[config.Batique]]
type = 'int'
name = 'zone_end'
value = 180
default = 180

[[config.Batique]]
type = 'float'
name = 'coord_scale'
value = 2.5
default = 2.5

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
