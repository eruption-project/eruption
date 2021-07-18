id = '5dc24fa6-e965-45cb-a0da-e87d28713095'
name = 'Psychedelic Twinkle'
description = 'The psychedelic twinkle profile'
active_scripts = [
	'psychedelic.lua',
	'shockwave.lua',
#	'impact.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Psychedelic"]]
type = 'float'
name = 'color_boost'
value = 25
default = 25

[[config."Psychedelic"]]
type = 'float'
name = 'saturation_boost'
value = 2
default = 2

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
