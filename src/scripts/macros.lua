-- This file is part of Eruption.

-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.

-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.

-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

require "declarations"
require "debug"

-- available modifier keys
CAPS_LOCK = 0
LEFT_SHIFT = 1
RIGHT_SHIFT = 2
LEFT_CTRL = 3
RIGHT_CTRL = 4
LEFT_ALT = 5
RIGHT_ALT = 6
RIGHT_MENU = 7

-- import user configuration
require "macros/modifiers"

-- initialize remapping tables
REMAPPING_TABLE = {}			-- level 1 remapping table (No modifier keys applied)

ACTIVE_EASY_SHIFT_LAYER = 1		-- level 4 supports up to 6 sub-layers
EASY_SHIFT_REMAPPING_TABLE = {  -- level 4 remapping table (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MACRO_TABLE = {	 	-- level 4 macro table (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}

-- import default color scheme
require "themes/default"

-- import custom macro definitions sub-modules
require(requires)

-- global state variables --
ticks = 0
color_map = {}

highlight_ttl = 0
highlight_max_ttl = 255

modifier_map = {} -- holds the state of modifier keys
game_mode_enabled = true  -- keyboard can be in "game mode" or in "normal mode";
						  -- until support for game mode is in place, just pretend
						  -- that we are in game mode, all the time

-- event handler functions --
function on_startup(config)
	modifier_map[CAPS_LOCK] = get_key_state(4)
	modifier_map[LEFT_SHIFT] = get_key_state(5)
	modifier_map[RIGHT_SHIFT] = get_key_state(83)
	modifier_map[LEFT_CTRL] = get_key_state(6)
	modifier_map[RIGHT_CTRL] = get_key_state(90)
	modifier_map[LEFT_ALT] = get_key_state(17)
	modifier_map[RIGHT_ALT] = get_key_state(71)
	modifier_map[RIGHT_MENU] = get_key_state(84)
end

function on_key_down(key_index)
	debug("Key down: Index: " .. key_index)
		
	-- update the modifier_map
	if key_index == 4 then 
		modifier_map[CAPS_LOCK] = true

		-- consume the CAPS_LOCK key while in game mode
 		if ENABLE_EASY_SHIFT and game_mode_enabled then 
			inject_key(0, false)
		end
	end

	if key_index == 5 then 
		modifier_map[LEFT_SHIFT] = true
	elseif key_index == 83 then
		modifier_map[RIGHT_SHIFT] = true
	elseif key_index == 6 then
		modifier_map[LEFT_CTRL] = true
	elseif key_index == 90 then
		modifier_map[RIGHT_CTRL] = true
	elseif key_index == 17 then
		modifier_map[LEFT_ALT] = true
	elseif key_index == 71 then
		modifier_map[RIGHT_ALT] = true
	elseif key_index == 84 then
		modifier_map[RIGHT_MENU] = true

		-- consume the menu key
		inject_key(0, false)
	end

	-- slot keys (F1 - F4)
	if modifier_map[MODIFIER_KEY] and key_index == 12 then
		do_switch_slot(0)
	elseif modifier_map[MODIFIER_KEY] and key_index == 18 then
		do_switch_slot(1)
	elseif modifier_map[MODIFIER_KEY] and key_index == 24 then
		do_switch_slot(2)
	elseif modifier_map[MODIFIER_KEY] and key_index == 29 then
		do_switch_slot(3)
	end

	-- macro keys (INSERT - PAGEDOWN)
	if modifier_map[MODIFIER_KEY] and key_index == 101 then
		on_macro_key_down(0)
	elseif modifier_map[MODIFIER_KEY] and key_index == 105 then
		on_macro_key_down(1)
	elseif modifier_map[MODIFIER_KEY] and key_index == 110 then
		on_macro_key_down(2)
	elseif modifier_map[MODIFIER_KEY] and key_index == 102 then
		on_macro_key_down(3)
	elseif modifier_map[MODIFIER_KEY] and key_index == 106 then
		on_macro_key_down(4)
	elseif modifier_map[MODIFIER_KEY] and key_index == 111 then
		on_macro_key_down(5)
	end

	-- switch Easy Shift+ layers via Caps Lock + macro keys
	if modifier_map[CAPS_LOCK] and key_index == 101 then
		do_switch_easy_shift_layer(0)
	elseif modifier_map[CAPS_LOCK] and key_index == 105 then
		do_switch_easy_shift_layer(1)
	elseif modifier_map[CAPS_LOCK] and key_index == 110 then
		do_switch_easy_shift_layer(2)
	elseif modifier_map[CAPS_LOCK] and key_index == 102 then
		do_switch_easy_shift_layer(3)
	elseif modifier_map[CAPS_LOCK] and key_index == 106 then
		do_switch_easy_shift_layer(4)
	elseif modifier_map[CAPS_LOCK] and key_index == 111 then
		do_switch_easy_shift_layer(5)
	end

	-- media keys (F9 - F12)
	-- if modifier_map[MODIFIER_KEY] and key_index == 79 then
	-- 	inject_key(165, true) -- EV_KEY::PREVSONG
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 85 then
	-- 	inject_key(166, true) -- EV_KEY::STOPCD
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 86 then
	-- 	inject_key(164, true) -- EV_KEY::PLAYPAUSE
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 87 then
	-- 	inject_key(163, true) -- EV_KEY::NEXTSONG
	-- end

	simple_remapping(key_index, true)

	-- call complex macros on the Easy Shift+ layer (layer 4)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and 
	    EASY_SHIFT_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][key_index] ~= nil then
		-- call associated function
		EASY_SHIFT_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][key_index]()
	end
end

function on_key_up(key_index)
	debug("Key up: Index: " .. key_index)

	-- update the modifier_map
	if key_index == 4 then 
		modifier_map[CAPS_LOCK] = false

		-- consume CAPS_LOCK key while in game mode
		if ENABLE_EASY_SHIFT and game_mode_enabled then 
			inject_key(0, false)
		end
	end

	if key_index == 5 then
		modifier_map[LEFT_SHIFT] = false
	elseif key_index == 83 then
		modifier_map[RIGHT_SHIFT] = false
	elseif key_index == 6 then
		modifier_map[LEFT_CTRL] = false
	elseif key_index == 90 then
		modifier_map[RIGHT_CTRL] = false
	elseif key_index == 17 then
		modifier_map[LEFT_ALT] = false
	elseif key_index == 71 then
		modifier_map[RIGHT_ALT] = false
	elseif key_index == 84 then
		modifier_map[RIGHT_MENU] = false

		-- consume menu key
		inject_key(0, false)
	end

	-- media keys (F9 - F12)
	-- if modifier_map[MODIFIER_KEY] and key_index == 79 then
	-- 	inject_key(165, false) -- EV_KEY::PREVSONG
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 85 then
	-- 	inject_key(166, false) -- EV_KEY::STOPCD
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 86 then
	-- 	inject_key(164, false) -- EV_KEY::PLAYPAUSE
	-- elseif modifier_map[MODIFIER_KEY] and key_index == 87 then
	-- 	inject_key(163, false) -- EV_KEY::NEXTSONG
	-- end

	simple_remapping(key_index, false)
end

-- perform a simple remapping
function simple_remapping(key_index, down)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT then
		code = EASY_SHIFT_REMAPPING_TABLE[ACTIVE_EASY_SHIFT_LAYER][key_index]
		if code ~= nil then
			inject_key(code, down)
		end
	else
		code = REMAPPING_TABLE[key_index]
		if code ~= nil then
			inject_key(code, down)
		end
	end
end

function do_switch_slot(index)
	debug("Switching to slot #" .. index + 1)

	-- consume the keystroke
	inject_key(0, false)

	-- tell the Eruption core to switch to a different slot
	switch_to_slot(index)
end

function do_switch_easy_shift_layer(index)
	debug("Switching to Easy Shift+ layer #" .. index + 1)

	-- consume the keystroke
	inject_key(0, false)

	ACTIVE_EASY_SHIFT_LAYER = index + 1
end

function on_tick(delta)
	ticks = ticks + delta + 1
	
	update_color_state()

	if highlight_ttl <= 0 then return end

    local num_keys = get_num_keys()

    -- show key highlight effect
    if ticks % animation_delay == 0 then
        for i = 0, num_keys do
			if highlight_ttl >= highlight_step then
				r, g, b, a = color_to_rgba(color_map[i])
				alpha = trunc(lerp(0, highlight_max_ttl / 255, highlight_ttl) * highlight_opacity)

				color_map[i] = rgba_to_color(r, g, b, min(255, alpha))
			else 
				highlight_ttl = 0
				color_map[i] = 0x00000000
			end
        end

		highlight_ttl = highlight_ttl - highlight_step

        submit_color_map(color_map)
    end
end
