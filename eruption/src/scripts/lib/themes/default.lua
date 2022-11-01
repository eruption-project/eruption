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
-- Copyright (c) 2019-2022, The Eruption Development Team
--
-- switch slots via F1-F4
COLOR_SWITCH_SLOT = rgb_to_color(32, 255, 255)
-- currently active slot
COLOR_ACTIVE_SLOT = rgb_to_color(128, 255, 255)

-- macro keys M1-M6
COLOR_MACRO_KEY = rgb_to_color(0, 255, 0)

-- F5-F12
COLOR_FUNCTION_KEY = rgb_to_color(32, 64, 128)
-- SCROLL LOCK, etc...
COLOR_FUNCTION_KEY_SPECIAL = rgb_to_color(255, 0, 0)

-- key remappings
COLOR_REMAPPED_KEY = rgb_to_color(0, 64, 255)
-- keys with complex macros
COLOR_ASSOCIATED_MACRO = rgb_to_color(0, 255, 0)

-- macro keys M1-M6 used to switch Easy Shift+ sub-layer
COLOR_SWITCH_EASY_SHIFT_LAYER = rgb_to_color(128, 8, 64)
COLOR_ACTIVE_EASY_SHIFT_LAYER = rgb_to_color(128, 128, 255)

-- indicators
COLOR_MUTE_AUDIO_MUTED = rgba_to_color(255, 0, 0, 255)
COLOR_MUTE_AUDIO_UNMUTED = rgba_to_color(0, 0, 0, 0)
COLOR_MUTE_AUDIO_OVERLAY = rgba_to_color(32, 32, 32, 32)
