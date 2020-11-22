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

function debug_print_led_state(color_map)
    info("************************************************************")

    local row = ""

    for i = 0, get_canvas_size() - 1 do

        if i % 21 == 0 then
            info(row)
            row = ""
        else
            local val = color_map[i]
            if val == nil then val = "nil" end

            row = row .. " | " .. val
        end
    end

    info("************************************************************")
end
