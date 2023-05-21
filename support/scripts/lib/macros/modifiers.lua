-- SPDX-License-Identifier: GPL-3.0-or-later
--
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
-- Copyright (c) 2019-2023, The Eruption Development Team
--
require "utilities"

-- specify whether the `Eruption Virtual Keyboard` is in charge of emitting events for the
-- function-, media-, volume/brightness and macro-keys.
-- Set this to `false` if the secondary USB sub-device is _not grabbed_ exclusively by Eruption,
-- therefor events originating from the sub-device will be handled by the window system.
-- Set this to `true` if the window system can't see the events of the sub-device because it is
-- grabbed exlusively by Eruption.
HANDLE_EXTRA_FUNCTIONS = true

ENABLE_EASY_SHIFT = true -- set this to false if you don't want to
-- use the Easy Shift+ functionality

ENABLE_SUPER_KEY_IN_GAME_MODE = false -- set this to true to enable the Windows key even when in game mode

-- comment out the declarations below to change the modifier key you want to use; default is the "FN" key:

MODIFIER_KEY = FN --
MODIFIER_KEY_INDEX = key_name_to_index("FN") -- the KEY_INDEX of the modifier key; has to match the key defined above
MODIFIER_KEY_EV_CODE = 464 -- the EV_KEY code of the modifier key; has to match the key defined above

-- or use this if you prefer "Right Meta" as the modifier key:

-- MODIFIER_KEY = RIGHT_META                               --
-- MODIFIER_KEY_INDEX = key_name_to_index("RIGHT_META")    -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 126                              -- the EV_KEY code of the modifier key; has to match the key defined above

-- or use this if you prefer "Right Menu" as the modifier key:

-- MODIFIER_KEY = RIGHT_MENU   				   			   --
-- MODIFIER_KEY_INDEX   = key_name_to_index("RIGHT_MENU")  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 127  				   			   -- the EV_KEY code of the modifier key; has to match the key defined above

-- or use this if you prefer "Right Alt" as the modifier key:

-- MODIFIER_KEY = RIGHT_ALT	   				   			   --
-- MODIFIER_KEY_INDEX   = key_name_to_index("RIGHT_ALT")   -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 100  				   			   -- the EV_KEY code of the modifier key; has to match the key defined above

-- or use this if you prefer "Right Shift" as the modifier key:

-- MODIFIER_KEY = RIGHT_SHIFT  				   			   --
-- MODIFIER_KEY_INDEX   = key_name_to_index("RIGHT_SHIFT") -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 54   				   			   -- the EV_KEY code of the modifier key; has to match the key defined above

-- or use this if you prefer "Right Ctrl" as the modifier key:

-- MODIFIER_KEY = RIGHT_CTRL   				   			   --
-- MODIFIER_KEY_INDEX   = key_name_to_index("RIGHT_CTRL")  -- the KEY_INDEX of the modifier key; has to match the key defined above
-- MODIFIER_KEY_EV_CODE = 97   				   			   -- the EV_KEY code of the modifier key; has to match the key defined above
