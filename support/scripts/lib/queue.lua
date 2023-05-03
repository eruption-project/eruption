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
queue = {}

function queue.new(max_size) return {first = 0, last = -1, max_size = max_size} end

function queue.size(list) return (list.last - list.first) + 1 end

function queue.max_size(list) return list.max_size end

function queue.push_left(list, value)
    local first = list.first - 1

    list.first = first
    list[first] = value

    if (list.last - list.first) + 1 > list.max_size then
        queue.pop_right(list)
    end
end

function queue.push_right(list, value)
    local last = list.last + 1

    list.last = last
    list[last] = value

    if (list.last - list.first) + 1 > list.max_size then queue.pop_left(list) end
end

function queue.pop_left(list)
    local first = list.first
    if first > list.last then
        -- error("queue is empty")
        return nil
    else
        local value = list[first]
        list[first] = nil -- to allow garbage collection
        list.first = first + 1

        return value
    end
end

function queue.pop_right(list)
    local last = list.last
    if list.first > last then
        -- error("queue is empty")
        return nil
    else
        local value = list[last]
        list[last] = nil -- to allow garbage collection
        list.last = last - 1

        return value
    end
end
