id = '5dc62fa6-e965-45cb-a0da-e87d29713059'
name = 'Red FX'
description = 'Red FX'
active_scripts = [
    'solid.lua',
#   'water.lua',
#   'phonon.lua',
    'halo.lua',
    'shockwave.lua',
    'impact.lua',
#   'ghost.lua',
#   'raindrops.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xffff000c

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.45

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false

[[config.Halo]]
type = 'color'
name = 'color_afterglow'
value = 0xffffaf00

[[config.Halo]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0x1affffff

[[config.Halo]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0x1affffff

# [[config.Halo]]
# type = 'float'
# name = 'opacity'
# value = 0.25

[[config.Shockwave]]
type = 'color'
name = 'color_shockwave'
value = 0xffff5f00

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0x80ffff00

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x000f0000
