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

-- nil safeguard utility function
function n(val)
    if val ~= nil then
        return val
    else
        -- debug("Returned 0 instead of nil")
        return 0
    end
end

-- nil safeguard utility function, with custom return value
function n2(val, ret)
    if val ~= nil then
        return val
    else
        -- debug("Returned ret instead of nil")
        return ret
    end
end

-- converts a key name to a key index
function key_name_to_index(name)
    if key_to_index ~= nil then
        local idx = key_to_index[name]
        if idx ~= nil then
            return idx
        else
            -- error("Could not find the index of key " .. name)
            return 0
        end
    else
        -- error("No supported hardware found, no device support scripts have been loaded")
        return 0
    end
end

-- returns the key index corresponding to the specified coordinates
function key_index(x, y)
    if x > max_keys_per_row or y > max_keys_per_col then
        error("Utilities: Coordinate out of bounds: x or y")
    end

    return n(rows_topology[22 * y + x]) + 1
end
