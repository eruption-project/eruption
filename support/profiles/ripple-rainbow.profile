id = '5dc62fa6-d865-55cb-a0da-e87d29713119'
name = 'Ripple: Rainbow'
description = 'Show rainbow colored waves on the press of a key'
active_scripts = [
    'solid.lua',
    'ripple.lua',
#   'impact.lua',
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
value = 0.20
default = 0.20

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
