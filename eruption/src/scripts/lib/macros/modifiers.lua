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

-- require "utilities"

if key_to_index == nil then
	error("No supported hardware found, no device support scripts have been loaded")
end

ENABLE_EASY_SHIFT = true   -- set this to false if you don't want to
						   -- use the Easy Shift+ functionality

ENABLE_SUPER_KEY_IN_GAME_MODE = false -- set this to true to enable the Windows key even when in game mode


-- comment out the declarations below to change the modifier key you want to use; default is the "FN" key:

MODIFIER_KEY = FN		       				   --
MODIFIER_KEY_INDEX   = key_to_index['FN']	   -- the KEY_INDEX of the modifier key; has to match the key defined above
MODIFIER_KEY_EV_CODE = 464     				   -- the EV_KEY code of the modifier key; has to match the key defined above


-- or use this if you prefer "Right Menu" as the modifier key:

-- MODIFIER_KEY = RIGHT_MENU   				   --
-- MODIFIER_KEY_INDEX   = key_to_index['RIGHT_MENU']  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 127  				   -- the EV_KEY code of the modifier key; has to match the key defined above


-- or use this if you prefer "Right Alt" as the modifier key:

-- MODIFIER_KEY = RIGHT_ALT	   				   --
-- MODIFIER_KEY_INDEX   = key_to_index['RIGHT_ALT']  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 100  				   -- the EV_KEY code of the modifier key; has to match the key defined above


-- or use this if you prefer "Right Shift" as the modifier key:

-- MODIFIER_KEY = RIGHT_SHIFT  				   --
-- MODIFIER_KEY_INDEX   = key_to_index['RIGHT_SHIFT']  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 54   				   -- the EV_KEY code of the modifier key; has to match the key defined above


-- or use this if you prefer "Right Ctrl" as the modifier key:

-- MODIFIER_KEY = RIGHT_CTRL   				   --
-- MODIFIER_KEY_INDEX   = key_to_index['RIGHT_CTRL']  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 97   				   -- the EV_KEY code of the modifier key; has to match the key defined above
