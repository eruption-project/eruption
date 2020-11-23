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

  for i = 0, num_keys do
      local key = "statistics.histograms." .. name .. "[" .. i .. "]"
      result[i] = load_int(key, 1)
  end

  return result
end

local function accum(key_histogram)
  local result = 0

  for i = 0, num_keys do
      result = result + key_histogram[i]
  end

  return result
end

-- event handler functions --
function on_tick(delta)
  local key_histogram = load_key_histogram(histogram_name)
  local sum_total = accum(key_histogram)

  for i = 0, num_keys do
    local percentile = ((key_histogram[i] * 10) / sum_total)
    color_map[i] = linear_gradient(color_cold, color_hot, percentile)
  end

  submit_color_map(color_map)
end
