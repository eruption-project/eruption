id = '5ac59fa6-e965-45cb-a0da-e87d29713106'
name = "Audio Visualization (4)"
description = "Audio Visualization effect"
active_scripts = [
	'domain-coloring.lua',
	'audioviz4.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Audio Visualizer #4"]]
type = 'float'
name = 'opacity'
value = 0.195
default = 0.195

[[config."Audio Visualizer #4"]]
type = 'color'
name = 'color_step'
value = 0xffffffff
default = 0xffffffff

[[config."Domain Coloring"]]
type = 'float'
name = 'color_divisor'
value = 1.0
default = 1.0

[[config."Domain Coloring"]]
type = 'float'
name = 'color_offset'
value = 0.0
default = 0.0

[[config."Domain Coloring"]]
type = 'float'
name = 'time_scale'
value = 50.0
default = 50.0

[[config."Domain Coloring"]]
type = 'float'
name = 'coord_scale'
value = 30.0
default = 30.0

[[config."Domain Coloring"]]
type = 'float'
name = 'opacity'
value = 1.0
default = 1.0

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
