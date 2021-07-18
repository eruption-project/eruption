id = '5dc62fa6-e965-55cb-a0da-e87d29714116'
name = 'Network FX'
description = 'Receive colors over a network socket'
active_scripts = [
    'netfx.lua',
    'shockwave.lua',
#   'impact.lua',
#   'water.lua',
#   'raindrops.lua',
#   'sysmon.lua',
#   'dim-zone.lua',
    'macros.lua',
#   'stats.lua',
]

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
