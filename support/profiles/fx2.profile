id = '5dc62fa6-e965-45cb-a0da-e87d29713106'
name = 'FX2'
description = 'Effects Profile #2'
active_scripts = [
    'solid.lua',
#   'impact.lua',
    'shockwave.lua',
#   'water.lua',
#   'raindrops.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffffffff
default = 0xffffffff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.25
default = 0.25

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5
default = 0.5

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
