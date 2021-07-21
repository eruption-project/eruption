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

function macro_inject_double_click_left()
    debug("UserMacros: macro_inject_double_click_left")

	macro_inject_double_click(1)
end

-- Inject a double click with the specified mouse button
function macro_inject_double_click(button_index)
    if MODIFIER_KEY ~= RIGHT_MENU then
        inject_key(MODIFIER_KEY_EV_CODE, false)  -- modifier key up
    end

    consume_key()

	inject_mouse_button(button_index, true)
    inject_mouse_button(button_index, false)

    delay(50)

    inject_mouse_button(button_index, true)
    inject_mouse_button(button_index, false)
end

function macro_insert_left_curly_bracket(down)
	debug("UserMacros: macro_insert_left_curly_bracket")

	if down then
		inject_key(100, true)	-- AltGr down
		inject_key(8, true)		-- 7/{ down
	else
		inject_key(8, false)	-- 7/{ up
		inject_key(100, false)	-- AltGr up
	end
end

function macro_insert_right_curly_bracket(down)
	debug("UserMacros: macro_insert_right_curly_bracket")

	if down then
		inject_key(100, true)	-- AltGr down
		inject_key(11, true)	-- 0/} down
	else
		inject_key(11, false)	-- 0/} up
		inject_key(100, false)	-- AltGr up
	end
end
