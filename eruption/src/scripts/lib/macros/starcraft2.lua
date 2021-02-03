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

-- This is the macro definitions file for the game "Star Craft 2".
-- You may want to customize the code below

-- override color scheme
require "themes/gaming"

function macro_select_idle_scvs()
	info("StarCraft2: Executing: Select idle SCVs")

	inject_key(29, true)	-- ctrl down
	inject_key(59, true)	-- F1 down

	inject_key(59, false)	-- F1 up
	inject_key(29, false)	-- ctrl up
end

function macro_select_all_military()
	info("StarCraft2: Executing: Select all military units")

	inject_key(29, true)	-- ctrl down
	inject_key(60, true)	-- F2 down

	inject_key(60, false)	-- F2 up
	inject_key(29, false)	-- ctrl up
end

function on_macro_key_down(index)
	info("StarCraft2: Executing: Macro #" .. index + 1)

	-- NOTE:
	-- We filter by slots, if you want to enable macros on all slots equally,
	-- just remove the 'and get_current_slot() == 0' part in each if statement.

	if index == 0 and get_current_slot() == 0 then
		if MODIFIER_KEY ~= RIGHT_MENU then
			inject_key(MODIFIER_KEY_EV_CODE, false)  -- modifier key up
		end

		-- consume the original keystroke
		consume_key()
	else
		-- no match, just consume the original keystroke and do nothing
		consume_key()
	end
end

function update_color_state()
	if ENABLE_EASY_SHIFT and game_mode_enabled and modifier_map[CAPS_LOCK] then
		-- Easy Shift+ key has been pressed

		-- highlight all keys
		for i = 0, canvas_size do color_map_highlight[i] = color_highlight end

		-- highlight remapped keys
		for i = 0, num_keys do
			if EASY_SHIFT_REMAPPING_TABLE[ACTIVE_EASY_SHIFT_LAYER][i] ~= nil then
				color_map_highlight[i] = COLOR_REMAPPED_KEY
			end
		end

		-- highlight keys with associated macros
		for i = 0, num_keys do
			if EASY_SHIFT_MACRO_TABLE[ACTIVE_EASY_SHIFT_LAYER][i] ~= nil then
				color_map_highlight[i] = COLOR_ASSOCIATED_MACRO
			end
		end

		-- Highlight Easy Shift+ key
		color_map_highlight[key_to_index['CAPS_LOCK']] = COLOR_FUNCTION_KEY_SPECIAL

		-- highlight the macro keys (INSERT - PAGEDOWN)
		color_map_highlight[key_to_index['INSERT']] = COLOR_SWITCH_EASY_SHIFT_LAYER
		color_map_highlight[key_to_index['POS1']] = COLOR_SWITCH_EASY_SHIFT_LAYER
		color_map_highlight[key_to_index['PGUP']] = COLOR_SWITCH_EASY_SHIFT_LAYER
		color_map_highlight[key_to_index['DEL']] = COLOR_SWITCH_EASY_SHIFT_LAYER
		color_map_highlight[key_to_index['END']] = COLOR_SWITCH_EASY_SHIFT_LAYER
		color_map_highlight[key_to_index['PGDWN']] = COLOR_SWITCH_EASY_SHIFT_LAYER

		-- highlight the active slot in a different color
		if ACTIVE_EASY_SHIFT_LAYER == 1 then
			color_map_highlight[key_to_index['INSERT']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		elseif ACTIVE_EASY_SHIFT_LAYER == 2 then
			color_map_highlight[key_to_index['POS1']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		elseif ACTIVE_EASY_SHIFT_LAYER == 3 then
			color_map_highlight[key_to_index['PGUP']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		elseif ACTIVE_EASY_SHIFT_LAYER == 4 then
			color_map_highlight[key_to_index['DEL']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		elseif ACTIVE_EASY_SHIFT_LAYER == 5 then
			color_map_highlight[key_to_index['END']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		elseif ACTIVE_EASY_SHIFT_LAYER == 6 then
			color_map_highlight[key_to_index['PGDWN']] = COLOR_ACTIVE_EASY_SHIFT_LAYER
		end

		highlight_ttl = highlight_max_ttl

	elseif modifier_map[MODIFIER_KEY] then
		-- modifier key has been pressed (eg: FN)

		-- highlight all keys
		for i = 0, canvas_size do color_map_highlight[i] = color_highlight end

		-- highlight the slot keys
		color_map_highlight[key_to_index['F1']] = COLOR_SWITCH_SLOT
		color_map_highlight[key_to_index['F2']] = COLOR_SWITCH_SLOT
		color_map_highlight[key_to_index['F3']] = COLOR_SWITCH_SLOT
		color_map_highlight[key_to_index['F4']] = COLOR_SWITCH_SLOT

		-- highlight the active slot in a different color
		if get_current_slot() == 0 then
			color_map_highlight[key_to_index['F1']] = COLOR_ACTIVE_SLOT
		elseif get_current_slot() == 1 then
			color_map_highlight[key_to_index['F2']] = COLOR_ACTIVE_SLOT
		elseif get_current_slot() == 2 then
			color_map_highlight[key_to_index['F3']] = COLOR_ACTIVE_SLOT
		elseif get_current_slot() == 3 then
			color_map_highlight[key_to_index['F4']] = COLOR_ACTIVE_SLOT
		end

		-- highlight function and media keys
		if MODIFIER_KEY == FN then
			if ENABLE_FUNCTION_KEYS then
				color_map_highlight[key_to_index['F5']] = COLOR_FUNCTION_KEY  -- F5 action
				color_map_highlight[key_to_index['F6']] = COLOR_FUNCTION_KEY  -- F6 action
				color_map_highlight[key_to_index['F7']] = COLOR_FUNCTION_KEY  -- F7 action
				color_map_highlight[key_to_index['F8']] = COLOR_FUNCTION_KEY  -- F8 action
			end

			if ENABLE_MEDIA_KEYS then
				color_map_highlight[key_to_index['F9']] = COLOR_FUNCTION_KEY  -- F9 action
				color_map_highlight[key_to_index['F10']] = COLOR_FUNCTION_KEY  -- F10 action
				color_map_highlight[key_to_index['F11']] = COLOR_FUNCTION_KEY  -- F11 action
				color_map_highlight[key_to_index['F12']] = COLOR_FUNCTION_KEY  -- F12 action
			end

			color_map_highlight[key_to_index['GAME_MODE']] = COLOR_FUNCTION_KEY_SPECIAL -- SCROLL LOCK/Game Mode

			if ENABLE_EASY_SHIFT and game_mode_enabled then
				color_map_highlight[key_to_index['CAPS_LOCK']] = COLOR_FUNCTION_KEY_SPECIAL -- Easy Shift+
			end
		end

		if ENABLE_MACRO_KEYS then
			-- highlight the macro keys (INSERT - PAGEDOWN)
			color_map_highlight[key_to_index['INSERT']] = COLOR_MACRO_KEY
			color_map_highlight[key_to_index['POS1']] = COLOR_MACRO_KEY
			color_map_highlight[key_to_index['PGUP']] = COLOR_MACRO_KEY
			color_map_highlight[key_to_index['DEL']] = COLOR_MACRO_KEY
			color_map_highlight[key_to_index['END']] = COLOR_MACRO_KEY
			color_map_highlight[key_to_index['PGDWN']] = COLOR_MACRO_KEY
		end

		highlight_ttl = highlight_max_ttl
	end

	-- in addition to the generic key highlighting, perform device specific key highlights
	device_specific_key_highlights()
end

-- remapping tables

-- change the tables below to perform a simple one-to-one mapping
-- convention is: REMAPPING_TABLE<[LAYER]>[KEY_INDEX] = EV_KEY_CONSTANT

-- find some examples below:
-- REMAPPING_TABLE[35]			    =  44  -- Remap: 'z' => 'y'

-- Enable the modifier key, while Easy Shift+ is activated
EASY_SHIFT_REMAPPING_TABLE[1][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE
EASY_SHIFT_REMAPPING_TABLE[2][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE
EASY_SHIFT_REMAPPING_TABLE[3][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE
EASY_SHIFT_REMAPPING_TABLE[4][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE
EASY_SHIFT_REMAPPING_TABLE[5][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE
EASY_SHIFT_REMAPPING_TABLE[6][MODIFIER_KEY_INDEX] = MODIFIER_KEY_EV_CODE

EASY_SHIFT_REMAPPING_TABLE[1][key_to_index['ESC']]  = 113  -- Remap: ESC => MUTE (audio), while Easy Shift+ is activated

-- assign macros to keys on the Easy Shift+ layer
EASY_SHIFT_MACRO_TABLE[1][key_to_index['1']]	= macro_select_idle_scvs  	 --
EASY_SHIFT_MACRO_TABLE[1][key_to_index['2']]	= macro_select_all_military  --
-- EASY_SHIFT_MACRO_TABLE[1][key_to_index['3']]	= easyshift_macro_3  	 	 --

-- assign macros to mouse buttons on the Easy Shift+ layer
-- EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[1][1]	= easyshift_mouse_macro_1  --
-- EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[1][2]	= easyshift_mouse_macro_2  --
-- EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[1][3]	= easyshift_mouse_macro_3  --

-- EASY_SHIFT_MOUSE_UP_MACRO_TABLE[1][1] = easyshift_mouse_macro_1  --
-- EASY_SHIFT_MOUSE_UP_MACRO_TABLE[1][2] = easyshift_mouse_macro_2  --
-- EASY_SHIFT_MOUSE_UP_MACRO_TABLE[1][3] = easyshift_mouse_macro_3  --

-- EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE[1][1] = easyshift_mouse_wheel_scroll_up  	--
-- EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE[1][2] = easyshift_mouse_wheel_scroll_down  	--

-- EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[1][1] = easyshift_mouse_dpi_changed  	--
-- EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[1][2] = easyshift_mouse_dpi_changed  	--
-- EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[1][3] = easyshift_mouse_dpi_changed  	--
-- EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[1][4] = easyshift_mouse_dpi_changed  	--
-- EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[1][5] = easyshift_mouse_dpi_changed  	--


-- ****************************************************************************
-- QWERTZ layout, column major order
-- Keyboard key   =>   KEY_INDEX table
-- ****************************************************************************
-- ESC			  =>   1
-- ^ 			  =>   2
-- TAB			  =>   3
-- CAPS LOCK	  =>   4
-- LEFT SHIFT	  =>   5
-- LEFT CTRL	  =>   6
-- 1			  =>   7
-- Q			  =>   8
-- A			  =>   9
-- <			  =>  10
-- LEFT SUPER	  =>  11
-- F1			  =>  12
-- 2			  =>  13
-- W			  =>  14
-- S			  =>  15
-- Y			  =>  16
-- LEFT ALT		  =>  17
-- F2			  =>  18
-- 3			  =>  19
-- E			  =>  20
-- D			  =>  21
-- X			  =>  22
-- <not assigned> =>  23
-- F3			  =>  24
-- 4			  =>  25
-- R			  =>  26
-- F			  =>  27
-- C			  =>  28
-- F4			  =>  29
-- 5			  =>  30
-- T			  =>  31
-- G			  =>  32
-- V			  =>  33
-- 6			  =>  34
-- Z			  =>  35
-- H			  =>  36
-- B			  =>  37
-- SPACE BAR	  =>  38
-- <not assigned> =>  39
-- <not assigned> =>  40
-- <not assigned> =>  41
-- <not assigned> =>  42
-- <not assigned> =>  43
-- <not assigned> =>  44
-- <not assigned> =>  45
-- <not assigned> =>  46
-- <not assigned> =>  47
-- <not assigned> =>  48
-- F5			  =>  49
-- 7			  =>  50
-- U			  =>  51
-- J			  =>  52
-- N			  =>  53
-- F6			  =>  54
-- 8			  =>  55
-- I			  =>  56
-- K			  =>  57
-- M			  =>  58
-- <not assigned> =>  59
-- F7			  =>  60
-- 9			  =>  61
-- O			  =>  62
-- L			  =>  63
-- ,;			  =>  64
-- <not assigned> =>  65
-- F8			  =>  66
-- 0			  =>  67
-- P			  =>  68
-- Ö			  =>  69
-- .:			  =>  70
-- RIGHT ALT	  =>  71
-- <not assigned> =>  72
-- ß			  =>  73
-- Ü			  =>  74
-- Ä			  =>  75
-- -			  =>  76
-- <not assigned> =>  77
-- <not assigned> =>  78
-- F9			  =>  79
-- ´			  =>  80
-- +*~			  =>  81
-- <not assigned> =>  82 -- #' ??
-- RIGHT SHIFT	  =>  83
-- RIGHT MENU	  =>  84
-- F10			  =>  85
-- F11			  =>  86
-- F12			  =>  87
-- BACKSPACE	  =>  88
-- RETURN		  =>  89
-- RIGHT CTRL	  =>  90
-- <not assigned> =>  91
-- <not assigned> =>  92
-- <not assigned> =>  93
-- <not assigned> =>  94
-- <not assigned> =>  95
-- <not assigned> =>  96
-- #'			  =>  97 -- ??
-- <not assigned> =>  98
-- <not assigned> =>  99
-- PRINT SCR	  => 100
-- INSERT		  => 101
-- DEL			  => 102
-- LEFT			  => 103
-- SCROLL LOCK	  => 104
-- HOME			  => 105
-- END			  => 106
-- UP			  => 107
-- DOWN			  => 108
-- PAUSE		  => 109
-- PAGE UP		  => 110
-- PAGE DOWN	  => 111
-- RIGHT		  => 112
-- <not assigned> => 113
-- NUM LOCK		  => 114
-- KP 7			  => 115
-- KP 4			  => 116
-- KP 1			  => 117
-- KP 0			  => 118
-- <not assigned> => 119
-- KP /			  => 120
-- KP 8			  => 121
-- KP 5			  => 122
-- KP 2			  => 123
-- <not assigned> => 124
-- LP *			  => 125
-- KP 9			  => 126
-- KP 6			  => 127
-- KP 3 		  => 128
-- KP ,			  => 129
-- KP -			  => 130
-- KP +			  => 131
-- KP ENTER		  => 132
-- <not assigned> => 133
-- <not assigned> => 134
-- <not assigned> => 135
-- <not assigned> => 136
-- <not assigned> => 137
-- <not assigned> => 138
-- <not assigned> => 139
-- <not assigned> => 140
-- <not assigned> => 141
-- <not assigned> => 142
-- <not assigned> => 143
-- <not assigned> => 144
-- ****************************************************************************

-- ****************************************************************************
-- Linux input subsystem EV_KEY definitions:
-- ****************************************************************************
-- 0 => EV_KEY::KEY_RESERVED
-- 1 => EV_KEY::KEY_ESC
-- 2 => EV_KEY::KEY_1
-- 3 => EV_KEY::KEY_2
-- 4 => EV_KEY::KEY_3
-- 5 => EV_KEY::KEY_4
-- 6 => EV_KEY::KEY_5
-- 7 => EV_KEY::KEY_6
-- 8 => EV_KEY::KEY_7
-- 9 => EV_KEY::KEY_8
-- 10 => EV_KEY::KEY_9
-- 11 => EV_KEY::KEY_0
-- 12 => EV_KEY::KEY_MINUS
-- 13 => EV_KEY::KEY_EQUAL
-- 14 => EV_KEY::KEY_BACKSPACE
-- 15 => EV_KEY::KEY_TAB
-- 16 => EV_KEY::KEY_Q
-- 17 => EV_KEY::KEY_W
-- 18 => EV_KEY::KEY_E
-- 19 => EV_KEY::KEY_R
-- 20 => EV_KEY::KEY_T
-- 21 => EV_KEY::KEY_Y
-- 22 => EV_KEY::KEY_U
-- 23 => EV_KEY::KEY_I
-- 24 => EV_KEY::KEY_O
-- 25 => EV_KEY::KEY_P
-- 26 => EV_KEY::KEY_LEFTBRACE
-- 27 => EV_KEY::KEY_RIGHTBRACE
-- 28 => EV_KEY::KEY_ENTER
-- 29 => EV_KEY::KEY_LEFTCTRL
-- 30 => EV_KEY::KEY_A
-- 31 => EV_KEY::KEY_S
-- 32 => EV_KEY::KEY_D
-- 33 => EV_KEY::KEY_F
-- 34 => EV_KEY::KEY_G
-- 35 => EV_KEY::KEY_H
-- 36 => EV_KEY::KEY_J
-- 37 => EV_KEY::KEY_K
-- 38 => EV_KEY::KEY_L
-- 39 => EV_KEY::KEY_SEMICOLON
-- 40 => EV_KEY::KEY_APOSTROPHE
-- 41 => EV_KEY::KEY_GRAVE
-- 42 => EV_KEY::KEY_LEFTSHIFT
-- 43 => EV_KEY::KEY_BACKSLASH
-- 44 => EV_KEY::KEY_Z
-- 45 => EV_KEY::KEY_X
-- 46 => EV_KEY::KEY_C
-- 47 => EV_KEY::KEY_V
-- 48 => EV_KEY::KEY_B
-- 49 => EV_KEY::KEY_N
-- 50 => EV_KEY::KEY_M
-- 51 => EV_KEY::KEY_COMMA
-- 52 => EV_KEY::KEY_DOT
-- 53 => EV_KEY::KEY_SLASH
-- 54 => EV_KEY::KEY_RIGHTSHIFT
-- 55 => EV_KEY::KEY_KPASTERISK
-- 56 => EV_KEY::KEY_LEFTALT
-- 57 => EV_KEY::KEY_SPACE
-- 58 => EV_KEY::KEY_CAPSLOCK
-- 59 => EV_KEY::KEY_F1
-- 60 => EV_KEY::KEY_F2
-- 61 => EV_KEY::KEY_F3
-- 62 => EV_KEY::KEY_F4
-- 63 => EV_KEY::KEY_F5
-- 64 => EV_KEY::KEY_F6
-- 65 => EV_KEY::KEY_F7
-- 66 => EV_KEY::KEY_F8
-- 67 => EV_KEY::KEY_F9
-- 68 => EV_KEY::KEY_F10
-- 69 => EV_KEY::KEY_NUMLOCK
-- 70 => EV_KEY::KEY_SCROLLLOCK
-- 71 => EV_KEY::KEY_KP7
-- 72 => EV_KEY::KEY_KP8
-- 73 => EV_KEY::KEY_KP9
-- 74 => EV_KEY::KEY_KPMINUS
-- 75 => EV_KEY::KEY_KP4
-- 76 => EV_KEY::KEY_KP5
-- 77 => EV_KEY::KEY_KP6
-- 78 => EV_KEY::KEY_KPPLUS
-- 79 => EV_KEY::KEY_KP1
-- 80 => EV_KEY::KEY_KP2
-- 81 => EV_KEY::KEY_KP3
-- 82 => EV_KEY::KEY_KP0
-- 83 => EV_KEY::KEY_KPDOT
-- 85 => EV_KEY::KEY_ZENKAKUHANKAKU
-- 86 => EV_KEY::KEY_102ND
-- 87 => EV_KEY::KEY_F11
-- 88 => EV_KEY::KEY_F12
-- 89 => EV_KEY::KEY_RO
-- 90 => EV_KEY::KEY_KATAKANA
-- 91 => EV_KEY::KEY_HIRAGANA
-- 92 => EV_KEY::KEY_HENKAN
-- 93 => EV_KEY::KEY_KATAKANAHIRAGANA
-- 94 => EV_KEY::KEY_MUHENKAN
-- 95 => EV_KEY::KEY_KPJPCOMMA
-- 96 => EV_KEY::KEY_KPENTER
-- 97 => EV_KEY::KEY_RIGHTCTRL
-- 98 => EV_KEY::KEY_KPSLASH
-- 99 => EV_KEY::KEY_SYSRQ
-- 100 => EV_KEY::KEY_RIGHTALT
-- 101 => EV_KEY::KEY_LINEFEED
-- 102 => EV_KEY::KEY_HOME
-- 103 => EV_KEY::KEY_UP
-- 104 => EV_KEY::KEY_PAGEUP
-- 105 => EV_KEY::KEY_LEFT
-- 106 => EV_KEY::KEY_RIGHT
-- 107 => EV_KEY::KEY_END
-- 108 => EV_KEY::KEY_DOWN
-- 109 => EV_KEY::KEY_PAGEDOWN
-- 110 => EV_KEY::KEY_INSERT
-- 111 => EV_KEY::KEY_DELETE
-- 112 => EV_KEY::KEY_MACRO
-- 113 => EV_KEY::KEY_MUTE
-- 114 => EV_KEY::KEY_VOLUMEDOWN
-- 115 => EV_KEY::KEY_VOLUMEUP
-- 116 => EV_KEY::KEY_POWER
-- 117 => EV_KEY::KEY_KPEQUAL
-- 118 => EV_KEY::KEY_KPPLUSMINUS
-- 119 => EV_KEY::KEY_PAUSE
-- 120 => EV_KEY::KEY_SCALE
-- 121 => EV_KEY::KEY_KPCOMMA
-- 122 => EV_KEY::KEY_HANGEUL
-- 123 => EV_KEY::KEY_HANJA
-- 124 => EV_KEY::KEY_YEN
-- 125 => EV_KEY::KEY_LEFTMETA
-- 126 => EV_KEY::KEY_RIGHTMETA
-- 127 => EV_KEY::KEY_COMPOSE
-- 128 => EV_KEY::KEY_STOP
-- 129 => EV_KEY::KEY_AGAIN
-- 130 => EV_KEY::KEY_PROPS
-- 131 => EV_KEY::KEY_UNDO
-- 132 => EV_KEY::KEY_FRONT
-- 133 => EV_KEY::KEY_COPY
-- 134 => EV_KEY::KEY_OPEN
-- 135 => EV_KEY::KEY_PASTE
-- 136 => EV_KEY::KEY_FIND
-- 137 => EV_KEY::KEY_CUT
-- 138 => EV_KEY::KEY_HELP
-- 139 => EV_KEY::KEY_MENU
-- 140 => EV_KEY::KEY_CALC
-- 141 => EV_KEY::KEY_SETUP
-- 142 => EV_KEY::KEY_SLEEP
-- 143 => EV_KEY::KEY_WAKEUP
-- 144 => EV_KEY::KEY_FILE
-- 145 => EV_KEY::KEY_SENDFILE
-- 146 => EV_KEY::KEY_DELETEFILE
-- 147 => EV_KEY::KEY_XFER
-- 148 => EV_KEY::KEY_PROG1
-- 149 => EV_KEY::KEY_PROG2
-- 150 => EV_KEY::KEY_WWW
-- 151 => EV_KEY::KEY_MSDOS
-- 152 => EV_KEY::KEY_COFFEE
-- 153 => EV_KEY::KEY_ROTATE_DISPLAY
-- 154 => EV_KEY::KEY_CYCLEWINDOWS
-- 155 => EV_KEY::KEY_MAIL
-- 156 => EV_KEY::KEY_BOOKMARKS
-- 157 => EV_KEY::KEY_COMPUTER
-- 158 => EV_KEY::KEY_BACK
-- 159 => EV_KEY::KEY_FORWARD
-- 160 => EV_KEY::KEY_CLOSECD
-- 161 => EV_KEY::KEY_EJECTCD
-- 162 => EV_KEY::KEY_EJECTCLOSECD
-- 163 => EV_KEY::KEY_NEXTSONG
-- 164 => EV_KEY::KEY_PLAYPAUSE
-- 165 => EV_KEY::KEY_PREVIOUSSONG
-- 166 => EV_KEY::KEY_STOPCD
-- 167 => EV_KEY::KEY_RECORD
-- 168 => EV_KEY::KEY_REWIND
-- 169 => EV_KEY::KEY_PHONE
-- 170 => EV_KEY::KEY_ISO
-- 171 => EV_KEY::KEY_CONFIG
-- 172 => EV_KEY::KEY_HOMEPAGE
-- 173 => EV_KEY::KEY_REFRESH
-- 174 => EV_KEY::KEY_EXIT
-- 175 => EV_KEY::KEY_MOVE
-- 176 => EV_KEY::KEY_EDIT
-- 177 => EV_KEY::KEY_SCROLLUP
-- 178 => EV_KEY::KEY_SCROLLDOWN
-- 179 => EV_KEY::KEY_KPLEFTPAREN
-- 180 => EV_KEY::KEY_KPRIGHTPAREN
-- 181 => EV_KEY::KEY_NEW
-- 182 => EV_KEY::KEY_REDO
-- 183 => EV_KEY::KEY_F13
-- 184 => EV_KEY::KEY_F14
-- 185 => EV_KEY::KEY_F15
-- 186 => EV_KEY::KEY_F16
-- 187 => EV_KEY::KEY_F17
-- 188 => EV_KEY::KEY_F18
-- 189 => EV_KEY::KEY_F19
-- 190 => EV_KEY::KEY_F20
-- 191 => EV_KEY::KEY_F21
-- 192 => EV_KEY::KEY_F22
-- 193 => EV_KEY::KEY_F23
-- 194 => EV_KEY::KEY_F24
-- 200 => EV_KEY::KEY_PLAYCD
-- 201 => EV_KEY::KEY_PAUSECD
-- 202 => EV_KEY::KEY_PROG3
-- 203 => EV_KEY::KEY_PROG4
-- 204 => EV_KEY::KEY_DASHBOARD
-- 205 => EV_KEY::KEY_SUSPEND
-- 206 => EV_KEY::KEY_CLOSE
-- 207 => EV_KEY::KEY_PLAY
-- 208 => EV_KEY::KEY_FASTFORWARD
-- 209 => EV_KEY::KEY_BASSBOOST
-- 210 => EV_KEY::KEY_PRINT
-- 211 => EV_KEY::KEY_HP
-- 212 => EV_KEY::KEY_CAMERA
-- 213 => EV_KEY::KEY_SOUND
-- 214 => EV_KEY::KEY_QUESTION
-- 215 => EV_KEY::KEY_EMAIL
-- 216 => EV_KEY::KEY_CHAT
-- 217 => EV_KEY::KEY_SEARCH
-- 218 => EV_KEY::KEY_CONNECT
-- 219 => EV_KEY::KEY_FINANCE
-- 220 => EV_KEY::KEY_SPORT
-- 221 => EV_KEY::KEY_SHOP
-- 222 => EV_KEY::KEY_ALTERASE
-- 223 => EV_KEY::KEY_CANCEL
-- 224 => EV_KEY::KEY_BRIGHTNESSDOWN
-- 225 => EV_KEY::KEY_BRIGHTNESSUP
-- 226 => EV_KEY::KEY_MEDIA
-- 227 => EV_KEY::KEY_SWITCHVIDEOMODE
-- 228 => EV_KEY::KEY_KBDILLUMTOGGLE
-- 229 => EV_KEY::KEY_KBDILLUMDOWN
-- 230 => EV_KEY::KEY_KBDILLUMUP
-- 231 => EV_KEY::KEY_SEND
-- 232 => EV_KEY::KEY_REPLY
-- 233 => EV_KEY::KEY_FORWARDMAIL
-- 234 => EV_KEY::KEY_SAVE
-- 235 => EV_KEY::KEY_DOCUMENTS
-- 236 => EV_KEY::KEY_BATTERY
-- 237 => EV_KEY::KEY_BLUETOOTH
-- 238 => EV_KEY::KEY_WLAN
-- 239 => EV_KEY::KEY_UWB
-- 240 => EV_KEY::KEY_UNKNOWN
-- 241 => EV_KEY::KEY_VIDEO_NEXT
-- 242 => EV_KEY::KEY_VIDEO_PREV
-- 243 => EV_KEY::KEY_BRIGHTNESS_CYCLE
-- 244 => EV_KEY::KEY_BRIGHTNESS_AUTO
-- 245 => EV_KEY::KEY_DISPLAY_OFF
-- 246 => EV_KEY::KEY_WWAN
-- 247 => EV_KEY::KEY_RFKILL
-- 248 => EV_KEY::KEY_MICMUTE
-- 352 => EV_KEY::KEY_OK
-- 353 => EV_KEY::KEY_SELECT
-- 354 => EV_KEY::KEY_GOTO
-- 355 => EV_KEY::KEY_CLEAR
-- 356 => EV_KEY::KEY_POWER2
-- 357 => EV_KEY::KEY_OPTION
-- 358 => EV_KEY::KEY_INFO
-- 359 => EV_KEY::KEY_TIME
-- 360 => EV_KEY::KEY_VENDOR
-- 361 => EV_KEY::KEY_ARCHIVE
-- 362 => EV_KEY::KEY_PROGRAM
-- 363 => EV_KEY::KEY_CHANNEL
-- 364 => EV_KEY::KEY_FAVORITES
-- 365 => EV_KEY::KEY_EPG
-- 366 => EV_KEY::KEY_PVR
-- 367 => EV_KEY::KEY_MHP
-- 368 => EV_KEY::KEY_LANGUAGE
-- 369 => EV_KEY::KEY_TITLE
-- 370 => EV_KEY::KEY_SUBTITLE
-- 371 => EV_KEY::KEY_ANGLE
-- 372 => EV_KEY::KEY_ZOOM
-- 373 => EV_KEY::KEY_MODE
-- 374 => EV_KEY::KEY_KEYBOARD
-- 375 => EV_KEY::KEY_SCREEN
-- 376 => EV_KEY::KEY_PC
-- 377 => EV_KEY::KEY_TV
-- 378 => EV_KEY::KEY_TV2
-- 379 => EV_KEY::KEY_VCR
-- 380 => EV_KEY::KEY_VCR2
-- 381 => EV_KEY::KEY_SAT
-- 382 => EV_KEY::KEY_SAT2
-- 383 => EV_KEY::KEY_CD
-- 384 => EV_KEY::KEY_TAPE
-- 385 => EV_KEY::KEY_RADIO
-- 386 => EV_KEY::KEY_TUNER
-- 387 => EV_KEY::KEY_PLAYER
-- 388 => EV_KEY::KEY_TEXT
-- 389 => EV_KEY::KEY_DVD
-- 390 => EV_KEY::KEY_AUX
-- 391 => EV_KEY::KEY_MP3
-- 392 => EV_KEY::KEY_AUDIO
-- 393 => EV_KEY::KEY_VIDEO
-- 394 => EV_KEY::KEY_DIRECTORY
-- 395 => EV_KEY::KEY_LIST
-- 396 => EV_KEY::KEY_MEMO
-- 397 => EV_KEY::KEY_CALENDAR
-- 398 => EV_KEY::KEY_RED
-- 399 => EV_KEY::KEY_GREEN
-- 400 => EV_KEY::KEY_YELLOW
-- 401 => EV_KEY::KEY_BLUE
-- 402 => EV_KEY::KEY_CHANNELUP
-- 403 => EV_KEY::KEY_CHANNELDOWN
-- 404 => EV_KEY::KEY_FIRST
-- 405 => EV_KEY::KEY_LAST
-- 406 => EV_KEY::KEY_AB
-- 407 => EV_KEY::KEY_NEXT
-- 408 => EV_KEY::KEY_RESTART
-- 409 => EV_KEY::KEY_SLOW
-- 410 => EV_KEY::KEY_SHUFFLE
-- 411 => EV_KEY::KEY_BREAK
-- 412 => EV_KEY::KEY_PREVIOUS
-- 413 => EV_KEY::KEY_DIGITS
-- 414 => EV_KEY::KEY_TEEN
-- 415 => EV_KEY::KEY_TWEN
-- 416 => EV_KEY::KEY_VIDEOPHONE
-- 417 => EV_KEY::KEY_GAMES
-- 418 => EV_KEY::KEY_ZOOMIN
-- 419 => EV_KEY::KEY_ZOOMOUT
-- 420 => EV_KEY::KEY_ZOOMRESET
-- 421 => EV_KEY::KEY_WORDPROCESSOR
-- 422 => EV_KEY::KEY_EDITOR
-- 423 => EV_KEY::KEY_SPREADSHEET
-- 424 => EV_KEY::KEY_GRAPHICSEDITOR
-- 425 => EV_KEY::KEY_PRESENTATION
-- 426 => EV_KEY::KEY_DATABASE
-- 427 => EV_KEY::KEY_NEWS
-- 428 => EV_KEY::KEY_VOICEMAIL
-- 429 => EV_KEY::KEY_ADDRESSBOOK
-- 430 => EV_KEY::KEY_MESSENGER
-- 431 => EV_KEY::KEY_DISPLAYTOGGLE
-- 432 => EV_KEY::KEY_SPELLCHECK
-- 433 => EV_KEY::KEY_LOGOFF
-- 434 => EV_KEY::KEY_DOLLAR
-- 435 => EV_KEY::KEY_EURO
-- 436 => EV_KEY::KEY_FRAMEBACK
-- 437 => EV_KEY::KEY_FRAMEFORWARD
-- 438 => EV_KEY::KEY_CONTEXT_MENU
-- 439 => EV_KEY::KEY_MEDIA_REPEAT
-- 440 => EV_KEY::KEY_10CHANNELSUP
-- 441 => EV_KEY::KEY_10CHANNELSDOWN
-- 442 => EV_KEY::KEY_IMAGES
-- 448 => EV_KEY::KEY_DEL_EOL
-- 449 => EV_KEY::KEY_DEL_EOS
-- 450 => EV_KEY::KEY_INS_LINE
-- 451 => EV_KEY::KEY_DEL_LINE
-- 464 => EV_KEY::KEY_FN
-- 465 => EV_KEY::KEY_FN_ESC
-- 466 => EV_KEY::KEY_FN_F1
-- 467 => EV_KEY::KEY_FN_F2
-- 468 => EV_KEY::KEY_FN_F3
-- 469 => EV_KEY::KEY_FN_F4
-- 470 => EV_KEY::KEY_FN_F5
-- 471 => EV_KEY::KEY_FN_F6
-- 472 => EV_KEY::KEY_FN_F7
-- 473 => EV_KEY::KEY_FN_F8
-- 474 => EV_KEY::KEY_FN_F9
-- 475 => EV_KEY::KEY_FN_F10
-- 476 => EV_KEY::KEY_FN_F11
-- 477 => EV_KEY::KEY_FN_F12
-- 478 => EV_KEY::KEY_FN_1
-- 479 => EV_KEY::KEY_FN_2
-- 480 => EV_KEY::KEY_FN_D
-- 481 => EV_KEY::KEY_FN_E
-- 482 => EV_KEY::KEY_FN_F
-- 483 => EV_KEY::KEY_FN_S
-- 484 => EV_KEY::KEY_FN_B
-- 497 => EV_KEY::KEY_BRL_DOT1
-- 498 => EV_KEY::KEY_BRL_DOT2
-- 499 => EV_KEY::KEY_BRL_DOT3
-- 500 => EV_KEY::KEY_BRL_DOT4
-- 501 => EV_KEY::KEY_BRL_DOT5
-- 502 => EV_KEY::KEY_BRL_DOT6
-- 503 => EV_KEY::KEY_BRL_DOT7
-- 504 => EV_KEY::KEY_BRL_DOT8
-- 505 => EV_KEY::KEY_BRL_DOT9
-- 506 => EV_KEY::KEY_BRL_DOT10
-- 512 => EV_KEY::KEY_NUMERIC_0
-- 513 => EV_KEY::KEY_NUMERIC_1
-- 514 => EV_KEY::KEY_NUMERIC_2
-- 515 => EV_KEY::KEY_NUMERIC_3
-- 516 => EV_KEY::KEY_NUMERIC_4
-- 517 => EV_KEY::KEY_NUMERIC_5
-- 518 => EV_KEY::KEY_NUMERIC_6
-- 519 => EV_KEY::KEY_NUMERIC_7
-- 520 => EV_KEY::KEY_NUMERIC_8
-- 521 => EV_KEY::KEY_NUMERIC_9
-- 522 => EV_KEY::KEY_NUMERIC_STAR
-- 523 => EV_KEY::KEY_NUMERIC_POUND
-- 524 => EV_KEY::KEY_NUMERIC_A
-- 525 => EV_KEY::KEY_NUMERIC_B
-- 526 => EV_KEY::KEY_NUMERIC_C
-- 527 => EV_KEY::KEY_NUMERIC_D
-- 528 => EV_KEY::KEY_CAMERA_FOCUS
-- 529 => EV_KEY::KEY_WPS_BUTTON
-- 530 => EV_KEY::KEY_TOUCHPAD_TOGGLE
-- 531 => EV_KEY::KEY_TOUCHPAD_ON
-- 532 => EV_KEY::KEY_TOUCHPAD_OFF
-- 533 => EV_KEY::KEY_CAMERA_ZOOMIN
-- 534 => EV_KEY::KEY_CAMERA_ZOOMOUT
-- 535 => EV_KEY::KEY_CAMERA_UP
-- 536 => EV_KEY::KEY_CAMERA_DOWN
-- 537 => EV_KEY::KEY_CAMERA_LEFT
-- 538 => EV_KEY::KEY_CAMERA_RIGHT
-- 539 => EV_KEY::KEY_ATTENDANT_ON
-- 540 => EV_KEY::KEY_ATTENDANT_OFF
-- 541 => EV_KEY::KEY_ATTENDANT_TOGGLE
-- 542 => EV_KEY::KEY_LIGHTS_TOGGLE
-- 560 => EV_KEY::KEY_ALS_TOGGLE
-- 561 => EV_KEY::KEY_ROTATE_LOCK_TOGGLE
-- 576 => EV_KEY::KEY_BUTTONCONFIG
-- 577 => EV_KEY::KEY_TASKMANAGER
-- 578 => EV_KEY::KEY_JOURNAL
-- 579 => EV_KEY::KEY_CONTROLPANEL
-- 580 => EV_KEY::KEY_APPSELECT
-- 581 => EV_KEY::KEY_SCREENSAVER
-- 582 => EV_KEY::KEY_VOICECOMMAND
-- 583 => EV_KEY::KEY_ASSISTANT
-- 592 => EV_KEY::KEY_BRIGHTNESS_MIN
-- 593 => EV_KEY::KEY_BRIGHTNESS_MAX
-- 608 => EV_KEY::KEY_KBDINPUTASSIST_PREV
-- 609 => EV_KEY::KEY_KBDINPUTASSIST_NEXT
-- 610 => EV_KEY::KEY_KBDINPUTASSIST_PREVGROUP
-- 611 => EV_KEY::KEY_KBDINPUTASSIST_NEXTGROUP
-- 612 => EV_KEY::KEY_KBDINPUTASSIST_ACCEPT
-- 613 => EV_KEY::KEY_KBDINPUTASSIST_CANCEL
-- 614 => EV_KEY::KEY_RIGHT_UP
-- 615 => EV_KEY::KEY_RIGHT_DOWN
-- 616 => EV_KEY::KEY_LEFT_UP
-- 617 => EV_KEY::KEY_LEFT_DOWN
-- 618 => EV_KEY::KEY_ROOT_MENU
-- 619 => EV_KEY::KEY_MEDIA_TOP_MENU
-- 620 => EV_KEY::KEY_NUMERIC_11
-- 621 => EV_KEY::KEY_NUMERIC_12
-- 622 => EV_KEY::KEY_AUDIO_DESC
-- 623 => EV_KEY::KEY_3D_MODE
-- 624 => EV_KEY::KEY_NEXT_FAVORITE
-- 625 => EV_KEY::KEY_STOP_RECORD
-- 626 => EV_KEY::KEY_PAUSE_RECORD
-- 627 => EV_KEY::KEY_VOD
-- 628 => EV_KEY::KEY_UNMUTE
-- 629 => EV_KEY::KEY_FASTREVERSE
-- 630 => EV_KEY::KEY_SLOWREVERSE
-- 631 => EV_KEY::KEY_DATA
-- 632 => EV_KEY::KEY_ONSCREEN_KEYBOARD
-- 767 => EV_KEY::KEY_MAX
-- 256 => EV_KEY::BTN_0
-- 257 => EV_KEY::BTN_1
-- 258 => EV_KEY::BTN_2
-- 259 => EV_KEY::BTN_3
-- 260 => EV_KEY::BTN_4
-- 261 => EV_KEY::BTN_5
-- 262 => EV_KEY::BTN_6
-- 263 => EV_KEY::BTN_7
-- 264 => EV_KEY::BTN_8
-- 265 => EV_KEY::BTN_9
-- 272 => EV_KEY::BTN_LEFT
-- 273 => EV_KEY::BTN_RIGHT
-- 274 => EV_KEY::BTN_MIDDLE
-- 275 => EV_KEY::BTN_SIDE
-- 276 => EV_KEY::BTN_EXTRA
-- 277 => EV_KEY::BTN_FORWARD
-- 278 => EV_KEY::BTN_BACK
-- 279 => EV_KEY::BTN_TASK
-- 288 => EV_KEY::BTN_TRIGGER
-- 289 => EV_KEY::BTN_THUMB
-- 290 => EV_KEY::BTN_THUMB2
-- 291 => EV_KEY::BTN_TOP
-- 292 => EV_KEY::BTN_TOP2
-- 293 => EV_KEY::BTN_PINKIE
-- 294 => EV_KEY::BTN_BASE
-- 295 => EV_KEY::BTN_BASE2
-- 296 => EV_KEY::BTN_BASE3
-- 297 => EV_KEY::BTN_BASE4
-- 298 => EV_KEY::BTN_BASE5
-- 299 => EV_KEY::BTN_BASE6
-- 303 => EV_KEY::BTN_DEAD
-- 304 => EV_KEY::BTN_SOUTH
-- 305 => EV_KEY::BTN_EAST
-- 306 => EV_KEY::BTN_C
-- 307 => EV_KEY::BTN_NORTH
-- 308 => EV_KEY::BTN_WEST
-- 309 => EV_KEY::BTN_Z
-- 310 => EV_KEY::BTN_TL
-- 311 => EV_KEY::BTN_TR
-- 312 => EV_KEY::BTN_TL2
-- 313 => EV_KEY::BTN_TR2
-- 314 => EV_KEY::BTN_SELECT
-- 315 => EV_KEY::BTN_START
-- 316 => EV_KEY::BTN_MODE
-- 317 => EV_KEY::BTN_THUMBL
-- 318 => EV_KEY::BTN_THUMBR
-- 320 => EV_KEY::BTN_TOOL_PEN
-- 321 => EV_KEY::BTN_TOOL_RUBBER
-- 322 => EV_KEY::BTN_TOOL_BRUSH
-- 323 => EV_KEY::BTN_TOOL_PENCIL
-- 324 => EV_KEY::BTN_TOOL_AIRBRUSH
-- 325 => EV_KEY::BTN_TOOL_FINGER
-- 326 => EV_KEY::BTN_TOOL_MOUSE
-- 327 => EV_KEY::BTN_TOOL_LENS
-- 328 => EV_KEY::BTN_TOOL_QUINTTAP
-- 329 => EV_KEY::BTN_STYLUS3
-- 330 => EV_KEY::BTN_TOUCH
-- 331 => EV_KEY::BTN_STYLUS
-- 332 => EV_KEY::BTN_STYLUS2
-- 333 => EV_KEY::BTN_TOOL_DOUBLETAP
-- 334 => EV_KEY::BTN_TOOL_TRIPLETAP
-- 335 => EV_KEY::BTN_TOOL_QUADTAP
-- 336 => EV_KEY::BTN_GEAR_DOWN
-- 337 => EV_KEY::BTN_GEAR_UP
-- 544 => EV_KEY::BTN_DPAD_UP
-- 545 => EV_KEY::BTN_DPAD_DOWN
-- 546 => EV_KEY::BTN_DPAD_LEFT
-- 547 => EV_KEY::BTN_DPAD_RIGHT
-- 704 => EV_KEY::BTN_TRIGGER_HAPPY1
-- 705 => EV_KEY::BTN_TRIGGER_HAPPY2
-- 706 => EV_KEY::BTN_TRIGGER_HAPPY3
-- 707 => EV_KEY::BTN_TRIGGER_HAPPY4
-- 708 => EV_KEY::BTN_TRIGGER_HAPPY5
-- 709 => EV_KEY::BTN_TRIGGER_HAPPY6
-- 710 => EV_KEY::BTN_TRIGGER_HAPPY7
-- 711 => EV_KEY::BTN_TRIGGER_HAPPY8
-- 712 => EV_KEY::BTN_TRIGGER_HAPPY9
-- 713 => EV_KEY::BTN_TRIGGER_HAPPY10
-- 714 => EV_KEY::BTN_TRIGGER_HAPPY11
-- 715 => EV_KEY::BTN_TRIGGER_HAPPY12
-- 716 => EV_KEY::BTN_TRIGGER_HAPPY13
-- 717 => EV_KEY::BTN_TRIGGER_HAPPY14
-- 718 => EV_KEY::BTN_TRIGGER_HAPPY15
-- 719 => EV_KEY::BTN_TRIGGER_HAPPY16
-- 720 => EV_KEY::BTN_TRIGGER_HAPPY17
-- 721 => EV_KEY::BTN_TRIGGER_HAPPY18
-- 722 => EV_KEY::BTN_TRIGGER_HAPPY19
-- 723 => EV_KEY::BTN_TRIGGER_HAPPY20
-- 724 => EV_KEY::BTN_TRIGGER_HAPPY21
-- 725 => EV_KEY::BTN_TRIGGER_HAPPY22
-- 726 => EV_KEY::BTN_TRIGGER_HAPPY23
-- 727 => EV_KEY::BTN_TRIGGER_HAPPY24
-- 728 => EV_KEY::BTN_TRIGGER_HAPPY25
-- 729 => EV_KEY::BTN_TRIGGER_HAPPY26
-- 730 => EV_KEY::BTN_TRIGGER_HAPPY27
-- 731 => EV_KEY::BTN_TRIGGER_HAPPY28
-- 732 => EV_KEY::BTN_TRIGGER_HAPPY29
-- 733 => EV_KEY::BTN_TRIGGER_HAPPY30
-- 734 => EV_KEY::BTN_TRIGGER_HAPPY31
-- 735 => EV_KEY::BTN_TRIGGER_HAPPY32
-- 736 => EV_KEY::BTN_TRIGGER_HAPPY33
-- 737 => EV_KEY::BTN_TRIGGER_HAPPY34
-- 738 => EV_KEY::BTN_TRIGGER_HAPPY35
-- 739 => EV_KEY::BTN_TRIGGER_HAPPY36
-- 740 => EV_KEY::BTN_TRIGGER_HAPPY37
-- 741 => EV_KEY::BTN_TRIGGER_HAPPY38
-- 742 => EV_KEY::BTN_TRIGGER_HAPPY39
-- 743 => EV_KEY::BTN_TRIGGER_HAPPY40
-- ****************************************************************************
