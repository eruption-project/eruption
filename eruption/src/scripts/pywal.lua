-- This file is part of Eruption.
--
-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
--
-- Copyright (c) 2019-2022, The Eruption Development Team
--
-- global array that stores each key's current color
--
json = require "lunajson"

file = io.open(os.getenv("HOME") .. "/.cache/wal/colors.json","r") -- Path to colors: $XDG_CACHE_HOME/wal/colors.json
jsonstr = file:read("*a") -- copy text to string
file:close()
colorsstr = json.decode(jsonstr) -- get color using "colorsstr["colors"]["color#"]" where "#" is an integar from 0 to 15

-- global state variables --
color_map = {}
ticks = 0

-- event handler functions --
function on_startup()
    for i = 1, get_canvas_size() do color_map[i] = 0x00000000 end
    submit_color_map(color_map)
end

function on_tick(delta)
    if not animate_gradient then return end
    ticks = ticks + delta

    --TODO: cycle between colors in the palette

    submit_color_map(color_map)
end