id = '5cd23fa6-e965-45cb-a0cd-e87d28713092'
name = 'Animal: Blobby + Color Swirls (Perlin)'
description = 'Simulate a lifeform with organic movements'
active_scripts = [
	'swirl-perlin.lua',
	'animal.lua',
    'afterhue.lua',
    'macros.lua',
#   'stats.lua',
]

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_divisor'
value = 2.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'color_offset'
value = -110.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'time_scale'
value = 250.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'coord_scale'
value = 15.0

[[config."Perlin Swirl"]]
type = 'float'
name = 'opacity'
value = 0.1

[[config.Animal]]
type = 'string'
name = 'name'
value = 'Blobby'

[[config.Animal]]
type = 'float'
name = 'opacity'
value = 1.0
