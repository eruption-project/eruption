id = '5dc62fa6-e965-45cb-a0da-e87d29713095'
name = 'Blue FX'
description = 'Default Profile #1'
active_scripts = [
    'solid.lua',
#   'water.lua',
#   'phonon.lua',
    'halo.lua',
    'shockwave.lua',
    'impact.lua',
#   'ghost.lua',
#   'raindrops.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff2020ff
default = 0xff2020ff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.45
default = 0.45

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0x80ffffff
default = 0x80ffffff

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5
default = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x0004040f
default = 0x0004040f

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
