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
require "declarations"
-- require "utilities"

-- available modifier keys
FN = 8

-- global state variables --
color_map = {}

modifier_map = {} -- holds the state of modifier keys
game_mode_enabled = load_bool_transient("global.game_mode_enabled", false) -- keyboard can be in "game mode" or in "normal mode";

-- utility functions --
function consume_key() inject_key(0, false) end

-- event handler functions --
function on_startup(config)
    for i = 1, canvas_size do color_map[i] = 0x00000000 end

    submit_color_map(color_map)
end

function on_hid_event(event_type, arg1)
    debug("Failsafe: HID event: " .. event_type .. " args: " .. arg1)

    local key_code = arg1

    local is_pressed = false
    if event_type == 2 then
        is_pressed = true
    elseif event_type == 1 then
        is_pressed = false
    end

    if key_code == FN_KEY then
        -- "FN" key event
        modifier_map[FN] = is_pressed

        debug("Failsafe: FN key event registered")
    elseif key_code == GAME_MODE_KEY then
        -- "SCROLL LOCK/GAME MODE" key event
        local fn_pressed = modifier_map[FN]
        if is_pressed and fn_pressed then
            game_mode_enabled = not game_mode_enabled
            store_bool_transient("global.game_mode_enabled", game_mode_enabled)

            debug("Failsafe: Game mode toggled")
        end
    end

    -- slot keys (F1 - F4) (force the use of FN key as modifier)
    if is_pressed then
        if modifier_map[FN] and key_code == 16 then
            do_switch_slot(0)
        elseif modifier_map[FN] and key_code == 24 then
            do_switch_slot(1)
        elseif modifier_map[FN] and key_code == 33 then
            do_switch_slot(2)
        elseif modifier_map[FN] and key_code == 32 then
            do_switch_slot(3)
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

        -- adjust volume
        if key_code == 1 then
            inject_key(114, true) -- VOLUME_DOWN (down)
            inject_key(114, false) -- VOLUME_DOWN (up)
        else
            inject_key(115, true) -- VOLUME_UP (down)
            inject_key(115, false) -- VOLUME_UP (up)
        end
    elseif event_type == 7 then
        -- Slot switching shortcuts
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

function on_key_down(key_index) debug("Failsafe: Key down: Index: " .. key_index) end

function on_key_up(key_index) debug("Failsafe: Key up: Index: " .. key_index) end

function do_switch_slot(index)
    debug("Failsafe: Switching to slot #" .. index + 1)

    -- consume the keystroke
    consume_key()

    -- tell the Eruption core to switch to a different slot
    switch_to_slot(index)
end
