id = '5dc62fa6-e965-45cb-a0da-e87d29713097'
name = "Spectrum Analyzer + Stripes"
description = "Default Profile #3"
active_scripts = [
	'stripes.lua',
	'audioviz3.lua',
	'afterglow.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config."Audio Visualizer #3 (Spectrum Analyzer)"]]
type = 'float'
name = 'opacity'
value = 0.90
default = 0.90

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
