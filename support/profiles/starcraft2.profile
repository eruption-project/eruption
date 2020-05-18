id = '5dc23fa9-e965-45cb-a0cd-e87d28713091'
name = 'Gaming: StarCraft 2'
description = 'The Star Craft 2 gaming profile'
active_scripts = [
    'solid.lua',
    'impact.lua',
    'shockwave.lua',
    'water.lua',
#   'raindrops.lua',
    'macros.lua',
]

[[config."Solid Color"]]
type = 'color'
name = 'color_background'
value = 0xff2020ff

[[config."Solid Color"]]
type = 'float'
name = 'opacity'
value = 0.35

[[config.Raindrops]]
type = 'float'
name = 'opacity'
value = 0.5

[[config.Macros]]
type = 'color'
name = 'color_highlight'
value = 0x0004040f

[[config.Macros]]
type = 'string'
name = 'requires'
value = 'macros/starcraft2'
