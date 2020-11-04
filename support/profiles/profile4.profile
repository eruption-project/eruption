id = '5dc62fa6-e965-45cb-a0da-e87d29713098'
name = "Linear Gradient"
description = "Default Profile #4"
active_scripts = [
	'linear-gradient.lua',
#	'impact.lua',
	'afterhue.lua',
 	'macros.lua',
#	'stats.lua',
]

[[config.Gradient]]
type = 'color'
name = 'color_start'
value = 0xffff0000

[[config.Gradient]]
type = 'color'
name = 'color_end'
value = 0xff0000ff

[[config.Gradient]]
type = 'int'
name = 'color_divisor'
value = 5

[[config.Gradient]]
type = 'bool'
name = 'animate_gradient'
value = true