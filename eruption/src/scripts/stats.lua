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
require "queue"
require "debug"

-- global state variables --
key_histogram = {}
key_histogram_errors = {}

key_ringbuffer = queue.new(15)

-- utility functions --
local function load_key_histogram(name)
    trace("Statistics: Loading histogram '" .. name .. "' from persistent storage")

    local result = {}

    for i = 0, num_keys do
        local key = "statistics.histograms." .. name .. "[" .. i .. "]"
        result[i] = load_int(key, 0)
    end

    return result
end

local function store_key_histogram(key_histogram, name)
    trace("Statistics: Saving histogram '" .. name .. "' to persistent storage")

    for i = 0, num_keys do
        local key = "statistics.histograms." .. name .. "[" .. i .. "]"
        store_int(key, key_histogram[i])
    end
end

local function dump_key_histogram(name)
    info("Statistics: Dumping: '" .. name .. "'")

    for i = 0, num_keys do
        local key = "statistics.histograms." .. name .. "[" .. i .. "]"
        local result = load_int(key, 0)

        info(i .. ": " .. result)
    end
end

-- event handler functions --
function on_startup(config)
    key_histogram = load_key_histogram("key_histogram")
    key_histogram_errors = load_key_histogram("key_histogram_errors")
end

function on_quit()
    store_key_histogram(key_histogram, "key_histogram")
    store_key_histogram(key_histogram_errors, "key_histogram_errors")
end

function on_key_down(key_index)
    trace("Statistics: Key down: " .. key_index)

    key_histogram[key_index] = key_histogram[key_index] + 1

    if key_index == 88 then
        -- backspace pressed
        local index = queue.pop_left(key_ringbuffer)

        if index ~= nil then
            key_histogram_errors[index] = key_histogram_errors[index] + 1
        end
    else
        -- other key pressed
        queue.push_left(key_ringbuffer, key_index)
    end
end

function on_key_up(key_index)
    trace("Statistics: Key up: " .. key_index)

    store_key_histogram(key_histogram, "key_histogram")
    store_key_histogram(key_histogram_errors, "key_histogram_errors")
end
