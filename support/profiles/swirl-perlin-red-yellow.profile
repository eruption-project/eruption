id = '5dc62fa6-e965-45cb-a0da-e87d29713118'
name = 'Color Swirls (Perlin): Red and Yellow'
description = 'Color Swirl effect'
active_scripts = [
    'swirl-perlin.lua',
    'halo.lua',
    'shockwave.lua',
#   'impact.lua',
#   'water.lua',
#   'raindrops.lua',
#   'sysmon.lua',
    'batique.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_divisor'
value = 10.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_offset'
value = 14.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'time_scale'
value = 250.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'coord_scale'
value = 15.0

[[config.Shockwave]]
type = 'color'
name = 'color_step_shockwave'
value = 0x05010000

[[config.Shockwave]]
type = 'bool'
name = 'mouse_events'
value = true

[[config.Shockwave]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0xa0ff0000

[[config.Shockwave]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0xd0ff0000

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.75

[[config."System Monitor"]]
type = 'color'
name = 'color_cold'
value = 0x0000ff00

[[config."System Monitor"]]
type = 'color'
name = 'color_hot'
value = 0xffff0000

# mouse support
[[config.Batique]]
type = 'int'
name = 'zone_start'
value = 144

[[config.Batique]]
type = 'int'
name = 'zone_end'
value = 180

[[config.Batique]]
type = 'float'
name = 'coord_scale'
value = 2.5

[[config.Batique]]
type = 'float'
name = 'color_divisor'
value = 10.0

[[config.Batique]]
type = 'float'
name = 'color_offset'
value = 14.0
