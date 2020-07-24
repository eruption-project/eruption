id = '5dc62fa6-e965-45cb-a0da-e87d29713093'
name = 'Organic FX'
description = 'Organic effects'
active_scripts = [
    'organic.lua',
    'shockwave.lua',
    'impact.lua',
#   'water.lua',
#   'raindrops.lua',
    'macros.lua',
    'stats.lua',
#   'profiles.lua',
]

[[config.Shockwave]]
type = 'color'
name = 'color_step_shockwave'
value = 0x05010000

[[config.Impact]]
type = 'bool'
name = 'mouse_events'
value = true

[[config.Impact]]
type = 'color'
name = 'color_mouse_click_flash'
value = 0xa0ff0000

[[config.Impact]]
type = 'color'
name = 'color_mouse_wheel_flash'
value = 0xd0ff0000

[[config.Water]]
type = 'float'
name = 'flow_speed'
value = 2.0

[[config.Water]]
type = 'float'
name = 'opacity'
value = 1.0

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.75
