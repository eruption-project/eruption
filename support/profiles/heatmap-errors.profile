id = '5dc62fa6-e965-45cb-a0da-e87d29713112'
name = "Heat Map (Typing Errors)"
description = "Display a heat map of previously recorded statistics"
active_scripts = [
	'heatmap.lua',
#   'dim-zone.lua',
 	'macros.lua',
 	'stats.lua',
]

[[config.Heatmap]]
type = 'string'
name = 'histogram_name'
value = 'key_histogram_errors'
default = 'key_histogram_errors'

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
