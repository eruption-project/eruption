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
require "debug"

-- global state variables --
color_map = {}

-- utility functions --
local function load_key_histogram(name)
  local result = {}
  local num_keys = get_num_keys()

  for i = 0, num_keys do
      local key = "statistics.histograms." .. name .. "[" .. i .. "]"
      result[i] = load_int(key, 0)
  end

  return result
end

local function find_max(key_histogram)
  local max = 0
  local num_keys = get_num_keys()

  for i = 0, num_keys do
      if key_histogram[i] > max then
        max = key_histogram[i]
      end
  end

  return max
end

-- event handler functions --
function on_tick(delta)
  local key_histogram = load_key_histogram("key_histogram")
  local max_val = find_max(key_histogram)

  local num_keys = get_num_keys()

  for i = 0, num_keys do
    color_map[i] = linear_gradient(color_cold, color_hot, lerp(0, max_val, ((key_histogram[i] / max_val) / max_val)))
  end

  submit_color_map(color_map)
end
