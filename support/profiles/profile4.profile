id = '5dc62fa6-e965-45cb-a0da-e87d29713098'
name = "Linear Gradient"
description = "Default Profile #4"
active_scripts = [
	'linear-gradient.lua',
#	'impact.lua',
	'afterhue.lua',
#   'dim-zone.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.Gradient]]
type = 'color'
name = 'color_start'
value = 0xffff0000
default = 0xffff0000

[[config.Gradient]]
type = 'color'
name = 'color_end'
value = 0xff0000ff
default = 0xff0000ff

[[config.Gradient]]
type = 'int'
name = 'color_divisor'
value = 5
default = 5

[[config.Gradient]]
type = 'bool'
name = 'animate_gradient'
value = true
default = true

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
