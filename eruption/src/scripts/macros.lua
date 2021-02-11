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
require "utilities"
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
FN = 8

-- import user configuration
require "macros/modifiers"

-- initialize remapping tables
REMAPPING_TABLE = {}				-- level 1 remapping table (No modifier keys applied)
MOUSE_HID_REMAPPING_TABLE = {}		-- level 1 remapping table for mouse events (No modifier keys applied)

ACTIVE_EASY_SHIFT_LAYER = 1			-- level 4 supports up to 6 sub-layers
EASY_SHIFT_REMAPPING_TABLE = {  	-- level 4 remapping table (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MACRO_TABLE = {	 		-- level 4 macro table (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}

EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE = {	-- macro tables for mouse button down events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MOUSE_UP_MACRO_TABLE = {		-- macro tables for mouse button up events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MOUSE_HID_DOWN_MACRO_TABLE = { -- macro tables for mouse (HID) button down events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MOUSE_HID_UP_MACRO_TABLE = { -- macro tables for mouse (HID) button up events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE = {	-- macro tables for mouse wheel events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}
EASY_SHIFT_MOUSE_DPI_MACRO_TABLE = {	-- macro tables for mouse DPI change events (Easy Shift+ layer)
	{}, {}, {}, {}, {}, {}
}

-- import default color scheme
require "themes/default"

-- import custom macro definitions sub-modules
require(requires)

-- global state variables --
ticks = 0
color_map = {}
color_map_highlight = {}
color_map_overlay = {}

max_effect_ttl = target_fps * 3
effect_ttl = max_effect_ttl

-- overlays
NO_OVERLAY = 0
VOLUME_OVERLAY = 1
BRIGHTNESS_OVERLAY = 2

overlay_state = NO_OVERLAY
overlay_ttl = 0
overlay_max_ttl = target_fps * 25

-- key highlighting
highlight_ttl = 0
highlight_max_ttl = 255

modifier_map = {} -- holds the state of modifier keys
game_mode_enabled = load_bool_transient("global.game_mode_enabled", false) -- keyboard can be in "game mode" or in "normal mode";

-- utility functions --
function consume_key()
	inject_key(0, false)
end

-- event handler functions --
function on_startup(config)
	modifier_map[CAPS_LOCK] = get_key_state(key_to_index['CAPS_LOCK'])
	modifier_map[LEFT_SHIFT] = get_key_state(key_to_index['LEFT_SHIFT'])
	modifier_map[RIGHT_SHIFT] = get_key_state(key_to_index['RIGHT_SHIFT'])
	modifier_map[LEFT_CTRL] = get_key_state(key_to_index['LEFT_CTRL'])
	modifier_map[RIGHT_CTRL] = get_key_state(key_to_index['RIGHT_CTRL'])
	modifier_map[LEFT_ALT] = get_key_state(key_to_index['LEFT_ALT'])
	modifier_map[RIGHT_ALT] = get_key_state(key_to_index['RIGHT_ALT'])
	modifier_map[RIGHT_MENU] = get_key_state(key_to_index['RIGHT_MENU'])
	modifier_map[FN] = get_key_state(key_to_index['FN'])

	for i = 0, canvas_size do
		color_map[i] = 0x00000000
		color_map_highlight[i] = 0x00000000
		color_map_overlay[i] = 0x00000000
	end
end

function on_hid_event(event_type, arg1)
	debug("Macros: HID event: " .. event_type .. " args: " .. arg1)

	local key_code = arg1

	local is_pressed = false
	if event_type == 2 then
		is_pressed = true
	elseif event_type == 1 then
		is_pressed = false
	end

	if key_code == 119 then
		-- "FN" key event
		modifier_map[FN] = is_pressed
	elseif key_code == EASY_SHIFT_KEY then
		-- "Easy Shift+" key event (CAPS LOCK pressed while in game mode)
		modifier_map[CAPS_LOCK] = is_pressed
	elseif key_code == GAME_MODE_KEY then
		-- "SCROLL LOCK/GAME MODE" key event
		local fn_pressed = modifier_map[FN]
		if is_pressed and fn_pressed then
			game_mode_enabled = not game_mode_enabled
			store_bool_transient("global.game_mode_enabled", game_mode_enabled)

			debug("Macros: Game mode toggled")
		end
	end

	-- special handling for keys in game mode
	if game_mode_enabled then
		-- handle the super key
		if ENABLE_SUPER_KEY_IN_GAME_MODE and key_code == 31 then
			if is_pressed then
				inject_key(125, true)  -- EV_KEY::LEFTMETA
			else
				inject_key(125, false) -- EV_KEY::LEFTMETA
			end
		end
	end

	-- slot keys (F1 - F4)
	if is_pressed then
		if modifier_map[MODIFIER_KEY] and key_code == 16 then
			do_switch_slot(0)
		elseif modifier_map[MODIFIER_KEY] and key_code == 24 then
			do_switch_slot(1)
		elseif modifier_map[MODIFIER_KEY] and key_code == 33 then
			do_switch_slot(2)
		elseif modifier_map[MODIFIER_KEY] and key_code == 32 then
			do_switch_slot(3)
		end
	end

	-- function keys (F5 - F8)
	if ENABLE_FUNCTION_KEYS then
		if is_pressed then
			if modifier_map[MODIFIER_KEY] and key_code == 40 then
				inject_key(144, true) -- EV_KEY::FILE
			elseif modifier_map[MODIFIER_KEY] and key_code == 48 then
				inject_key(172, true) -- EV_KEY::HOMEPAGE
			elseif modifier_map[MODIFIER_KEY] and key_code == 56 then
				inject_key(155, true) -- EV_KEY::MAIL
			elseif modifier_map[MODIFIER_KEY] and key_code == 57 then
				inject_key(140, true) -- EV_KEY::CALC
			end
		else
			if modifier_map[MODIFIER_KEY] and key_code == 40 then
				inject_key(144, false) -- EV_KEY::FILE
			elseif modifier_map[MODIFIER_KEY] and key_code == 48 then
				inject_key(172, false) -- EV_KEY::HOMEPAGE
			elseif modifier_map[MODIFIER_KEY] and key_code == 56 then
				inject_key(155, false) -- EV_KEY::MAIL
			elseif modifier_map[MODIFIER_KEY] and key_code == 57 then
				inject_key(140, false) -- EV_KEY::CALC
			end
		end
	end

	-- media keys (F9 - F12)
	if ENABLE_MEDIA_KEYS then
		if is_pressed then
			if modifier_map[MODIFIER_KEY] and key_code == 64 then
				inject_key(165, true) -- EV_KEY::PREVSONG
			elseif modifier_map[MODIFIER_KEY] and key_code == 72 then
				inject_key(166, true) -- EV_KEY::STOPCD
			elseif modifier_map[MODIFIER_KEY] and key_code == 80 then
				inject_key(164, true) -- EV_KEY::PLAYPAUSE
			elseif modifier_map[MODIFIER_KEY] and key_code == 81 then
				inject_key(163, true) -- EV_KEY::NEXTSONG
			end
		else
			if modifier_map[MODIFIER_KEY] and key_code == 64 then
				inject_key(165, false) -- EV_KEY::PREVSONG
			elseif modifier_map[MODIFIER_KEY] and key_code == 72 then
				inject_key(166, false) -- EV_KEY::STOPCD
			elseif modifier_map[MODIFIER_KEY] and key_code == 80 then
				inject_key(164, false) -- EV_KEY::PLAYPAUSE
			elseif modifier_map[MODIFIER_KEY] and key_code == 81 then
				inject_key(163, false) -- EV_KEY::NEXTSONG
			end
		end
	end

	-- process other HID events
	if event_type == 3 then
		-- Mute button event
		if key_code == 1 then
			inject_key(113, true) -- KEY_MUTE (audio) (down)
		else
			inject_key(113, false) -- KEY_MUTE (audio) (up)
		end
	elseif event_type == 4 then
		-- Volume dial knob rotation
		local event_handled = false
		if on_dial_knob_rotate_left ~= nil and on_dial_knob_rotate_right ~= nil then
			if key_code == 0 then
				-- default behaviour may be overridden by a user macro
				event_handled = on_dial_knob_rotate_right(key_code)
			else
				-- default behaviour may be overridden by a user macro
				event_handled = on_dial_knob_rotate_left(key_code)
			end
		end

		if not event_handled then
			-- adjust volume
			overlay_state = VOLUME_OVERLAY
			overlay_ttl = overlay_max_ttl

			if key_code == 1 then
				inject_key(114, true) -- VOLUME_DOWN (down)
				inject_key(114, false) -- VOLUME_DOWN (up)
			else
				inject_key(115, true) -- VOLUME_UP (down)
				inject_key(115, false) -- VOLUME_UP (up)
			end
		end
	elseif event_type == 5 then
		-- Brightness dial knob rotation or brightness shortcut pressed
		local event_handled = false
		if on_dial_knob_rotate_left ~= nil and on_dial_knob_rotate_right ~= nil then
			if key_code == 0 then
				-- default behaviour may be overridden by a user macro
				event_handled = on_dial_knob_rotate_right(key_code)
			else
				-- default behaviour may be overridden by a user macro
				event_handled = on_dial_knob_rotate_left(key_code)
			end
		end

		if not event_handled then
			-- adjust brightness
			-- overlay_state = NO_OVERLAY
			-- overlay_ttl = overlay_max_ttl

			local brightness = get_brightness()

			if key_code == 1 then
				brightness = brightness - 5
			else
				brightness = brightness + 5
			end

			brightness = clamp(brightness, 0, 100)

			set_brightness(brightness)
		end
	elseif event_type == 7 then
		-- Slot switching shortcuts
		local event_handled = false
		if on_previous_slot ~= nil and on_next_slot ~= nil then
			if key_code == 0 then
				-- default behaviour may be overridden by a user macro
				event_handled = on_previous_slot(key_code)
			else
				-- default behaviour may be overridden by a user macro
				event_handled = on_next_slot(key_code)
			end
		end

		if not event_handled then
			local current_slot = get_current_slot()

			if key_code == 0 then
				if current_slot - 1 >= 0 then
					do_switch_slot(current_slot - 1)
				end
			else
				if current_slot + 1 < 4 then
					do_switch_slot(current_slot + 1)
				end
			end
		end
	end
end

function on_mouse_hid_event(event_type, arg1)
	debug("Macros: HID event (mouse): " .. event_type .. " args: " .. arg1)

	if event_type == 1 then
		-- DPI change event
		local dpi_slot = arg1

		if game_mode_enabled then
			-- call complex macros on the Easy Shift+ layer (layer 4)
			if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and
				EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][dpi_slot] ~= nil then

				-- call associated function
				EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][dpi_slot]()
			end
		end
	elseif event_type == 2 then
		-- Button down event
		local button_index = arg1

		if game_mode_enabled then
			-- call complex macros on the Easy Shift+ layer (layer 4)
			if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and
				EASY_SHIFT_MOUSE_HID_DOWN_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index] ~= nil then

				-- call associated function
				EASY_SHIFT_MOUSE_HID_DOWN_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index]()
			end
		else
			simple_mouse_remapping(button_index, true)
		end
	elseif event_type == 3 then
		-- Button up event
		local button_index = arg1

		if game_mode_enabled then
			-- call complex macros on the Easy Shift+ layer (layer 4)
			if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and
				EASY_SHIFT_MOUSE_HID_UP_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index] ~= nil then

				-- call associated function
				EASY_SHIFT_MOUSE_HID_UP_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index]()
			end
		else
			simple_mouse_remapping(button_index, false)
		end
	end
end

function on_key_down(key_index)
	debug("Macros: Key down: Index: " .. key_index)

	-- update the modifier_map
	if key_index == key_to_index['CAPS_LOCK'] then
		modifier_map[CAPS_LOCK] = true

		-- consume the CAPS_LOCK key while in game mode
 		if ENABLE_EASY_SHIFT and game_mode_enabled then
			consume_key()
		end
	end

	if key_index == key_to_index['LEFT_SHIFT'] then
		modifier_map[LEFT_SHIFT] = true
	elseif key_index == key_to_index['RIGHT_SHIFT'] then
		modifier_map[RIGHT_SHIFT] = true
	elseif key_index == key_to_index['LEFT_CTRL'] then
		modifier_map[LEFT_CTRL] = true
	elseif key_index == key_to_index['RIGHT_CTRL'] then
		modifier_map[RIGHT_CTRL] = true
	elseif key_index == key_to_index['LEFT_ALT'] then
		modifier_map[LEFT_ALT] = true
	elseif key_index == key_to_index['RIGHT_ALT'] then
		modifier_map[RIGHT_ALT] = true
	elseif key_index == key_to_index['RIGHT_MENU'] then
		modifier_map[RIGHT_MENU] = true

		if MODIFIER_KEY == RIGHT_MENU then
			-- consume the menu key
			consume_key()
		end
	end

	-- slot keys (F1 - F4)
	if modifier_map[MODIFIER_KEY] and key_index == key_to_index['F1'] then
		do_switch_slot(0)
	elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['F2'] then
		do_switch_slot(1)
	elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['F3'] then
		do_switch_slot(2)
	elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['F4'] then
		do_switch_slot(3)
	end

	-- macro keys (INSERT - PAGEDOWN)
	if ENABLE_MACRO_KEYS then
		if modifier_map[MODIFIER_KEY] and key_index == key_to_index['INSERT'] then
			on_macro_key_down(0)
		elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['POS1'] then
			on_macro_key_down(1)
		elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['PGUP'] then
			on_macro_key_down(2)
		elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['DEL'] then
			on_macro_key_down(3)
		elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['END'] then
			on_macro_key_down(4)
		elseif modifier_map[MODIFIER_KEY] and key_index == key_to_index['PGDWN'] then
			on_macro_key_down(5)
		end
	end

	-- switch Easy Shift+ layers via Caps Lock + macro keys
	if modifier_map[CAPS_LOCK] and key_index == key_to_index['INSERT'] then
		do_switch_easy_shift_layer(0)
	elseif modifier_map[CAPS_LOCK] and key_index == key_to_index['POS1'] then
		do_switch_easy_shift_layer(1)
	elseif modifier_map[CAPS_LOCK] and key_index == key_to_index['PGUP'] then
		do_switch_easy_shift_layer(2)
	elseif modifier_map[CAPS_LOCK] and key_index == key_to_index['DEL'] then
		do_switch_easy_shift_layer(3)
	elseif modifier_map[CAPS_LOCK] and key_index == key_to_index['END'] then
		do_switch_easy_shift_layer(4)
	elseif modifier_map[CAPS_LOCK] and key_index == key_to_index['PGDWN'] then
		do_switch_easy_shift_layer(5)
	end

	simple_remapping(key_index, true)

	if game_mode_enabled then
		-- call complex macros on the Easy Shift+ layer (layer 4)
		if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and
			EASY_SHIFT_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][key_index] ~= nil then
			-- consume the original key press
			consume_key()

			-- call associated function
			EASY_SHIFT_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][key_index]()
		end
	end
end

function on_key_up(key_index)
	debug("Macros: Key up: Index: " .. key_index)

	-- update the modifier_map
	if key_index == key_to_index['CAPS_LOCK'] then
		modifier_map[CAPS_LOCK] = false

		-- consume CAPS_LOCK key while in game mode
		if ENABLE_EASY_SHIFT and game_mode_enabled then
			consume_key()
		end
	end

	if key_index == key_to_index['LEFT_SHIFT'] then
		modifier_map[LEFT_SHIFT] = false
	elseif key_index == key_to_index['RIGHT_SHIFT'] then
		modifier_map[RIGHT_SHIFT] = false
	elseif key_index == key_to_index['LEFT_CTRL'] then
		modifier_map[LEFT_CTRL] = false
	elseif key_index == key_to_index['RIGHT_CTRL'] then
		modifier_map[RIGHT_CTRL] = false
	elseif key_index == key_to_index['LEFT_ALT'] then
		modifier_map[LEFT_ALT] = false
	elseif key_index == key_to_index['RIGHT_ALT'] then
		modifier_map[RIGHT_ALT] = false
	elseif key_index == key_to_index['RIGHT_MENU'] then
		modifier_map[RIGHT_MENU] = false

		if MODIFIER_KEY == RIGHT_MENU then
			-- consume the menu key
			consume_key()
		end
	end

	simple_remapping(key_index, false)
end

function on_mouse_button_down(button_index)
	debug("Macros: Mouse down: Button: " .. button_index)

	-- call complex macros on the Easy Shift+ layer (layer 4)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and game_mode_enabled and
		EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index] ~= nil then
		-- consume the original mouse click
		inject_mouse_button(0, false)

		-- call associated function
		EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index]()
	end
end

function on_mouse_button_up(button_index)
	debug("Macros: Mouse up: Button: " .. button_index)

	-- call complex macros on the Easy Shift+ layer (layer 4)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and game_mode_enabled and
		EASY_SHIFT_MOUSE_UP_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index] ~= nil then
		-- consume the original mouse click
		inject_mouse_button(0, false)

		-- call associated function
		EASY_SHIFT_MOUSE_UP_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index]()
	end
end

function on_mouse_wheel(direction)
	debug("Macros: Mouse wheel: Direction: " .. direction)

	-- call complex macros on the Easy Shift+ layer (layer 4)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT and game_mode_enabled and
		EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][direction] ~= nil then
		-- consume the original mouse wheel event
		inject_mouse_wheel(0)

		-- call associated function
		EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][direction]()
	end
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

-- perform a simple remapping (for mouse events)
function simple_mouse_remapping(button_index, down)
	if modifier_map[CAPS_LOCK] and ENABLE_EASY_SHIFT then
		code = EASY_SHIFT_MOUSE_HID_REMAPPING_TABLE[ACTIVE_EASY_SHIFT_LAYER][button_index]
		if code ~= nil then
			inject_mouse_button(code, down)
		end
	else
		code = MOUSE_HID_REMAPPING_TABLE[button_index]
		if code ~= nil then
			inject_mouse_button(code, down)
		end
	end
end

function do_switch_slot(index)
	debug("Macros: Switching to slot #" .. index + 1)

	-- consume the keystroke
	consume_key()

	-- tell the Eruption core to switch to a different slot
	switch_to_slot(index)
end

function do_switch_easy_shift_layer(index)
	debug("Macros: Switching to Easy Shift+ layer #" .. index + 1)

	-- consume the keystroke
	consume_key()

	ACTIVE_EASY_SHIFT_LAYER = index + 1
end

function update_overlay_state()
	if overlay_state == NO_OVERLAY then
		overlay_ttl = 0
	elseif overlay_state == VOLUME_OVERLAY then
		-- generate color map values
		local percentage = get_audio_volume()
		local highlight_columns = (num_cols + 1) * percentage / 100

		-- compute which keys to highlight
		local upper_bound = 1
		for i = 1, highlight_columns do
			upper_bound = upper_bound + keys_per_col[i]
		end

		-- fill background
		for i = 1, num_keys do
			color_map_overlay[i] = rgb_to_color(16, 16, 16)
		end

		-- render volume level as highlight
		for i = 1, num_keys do
			if i < upper_bound then
				color_map_overlay[i] = rgb_to_color(255, 255, 255)
			end
		end
	end
end

function on_tick(delta)
	ticks = ticks + delta

	update_color_state()
	update_overlay_state()

	if effect_ttl <= 0 and highlight_ttl <= 0 and overlay_ttl <= 0 then return end

    -- show key highlight effect or the active overlay
	if ticks % animation_delay == 0 then
		for i = 0, num_keys do
			-- key highlight effect
			if highlight_ttl > 0 then
				r, g, b, a = color_to_rgba(color_map_highlight[i])
				alpha = range(0, 255, 0, highlight_max_ttl, highlight_ttl)

				color_map_highlight[i] = rgba_to_color(r, g, b, min(255, alpha))
				color_map[i] = color_map_highlight[i]
			else
				highlight_ttl = 0
				color_map_highlight[i] = 0x00000000
			end

			-- overlay effect
			if overlay_ttl > 0 then
				r, g, b, a = color_to_rgba(color_map_overlay[i])
				alpha = range(0, 255, 0, overlay_max_ttl, overlay_ttl)

				color_map_overlay[i] = rgba_to_color(r, g, b, min(255, alpha))
				color_map[i] = color_map_overlay[i]
			else
				overlay_ttl = 0
				color_map_overlay[i] = 0x00000000
			end
        end

		-- apply easing
		highlight_ttl = highlight_ttl - max(1.0, ((highlight_ttl / highlight_step) * 4))
		overlay_ttl = overlay_ttl - max(1.0, ((overlay_ttl / overlay_step) / 2))

		if overlay_ttl <= 0 then
			overlay_state = NO_OVERLAY
		end

		effect_ttl = effect_ttl - 1

		submit_color_map(color_map)
    end
end
