id = '5dc62fa8-e969-45cb-a0cd-e87d29713095'
name = 'Blue FX + Color Swirls (Perlin)'
description = 'Blue FX Effect + Color Swirls (Perlin)'
active_scripts = [
    'swirl-perlin.lua',
    'solid.lua',
#   'water.lua',
#   'phonon.lua',
    'halo.lua',
#   'shockwave.lua',
    'impact.lua',
#   'ghost.lua',
#   'raindrops.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_divisor'
value = 1.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_offset'
value = 0.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'time_scale'
value = 150.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'coord_scale'
value = 14.0

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff2020ff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.70

# [[config.Halo]]
# type = 'bool'
# name = 'mouse_events'
# value = false

[[config.Impact]]
type = 'color'
name = 'color_impact'
value = 0x80ffffff

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x0004040f
