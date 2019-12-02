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

-- global state variables --
color_map = {}
color_map_pressed = {}
color_map_water = {}

-- compute water with a supersampling factor of 8
water_grid_rows = 6 * 8
water_grid_cols = 22 * 8

-- holds a vector flow field to simulate water
water_grid = {}
color_palette = {}

ticks = 0

-- event handler functions --
function on_startup(config)
    init_state()
end

function on_quit(exit_code)
    init_state()
    set_color_map(color_map)
end

function on_key_down(key_index)
    color_map_pressed[key_index] = color_afterglow
    water_grid[key_index] = rand(127, 255)
end

function compute_waterfall(ticks)
    local num_keys = get_num_keys()

    if ticks % flow_speed == 0 then
        -- randomize bottom row
        for x = 0, water_grid_cols - 1 do
            water_grid[water_grid_rows * x + water_grid_cols] = rand(55, 255)
        end

        -- compute water from top to bottom
        for y = 0, water_grid_rows - 2 do
            for x = 0, water_grid_cols - 1 do
                water_grid[y * water_grid_cols + x] = 
                                ((water_grid[((y + 1) % water_grid_rows) * water_grid_cols + ((x - 1 + water_grid_cols) % water_grid_cols)]
                                + water_grid[((y + 1) % water_grid_rows) * water_grid_cols + ((x) % water_grid_cols)]
                                + water_grid[((y + 1) % water_grid_rows) * water_grid_cols + ((x + 1) % water_grid_cols)]
                                + water_grid[((y + 2) % water_grid_rows) * water_grid_cols + ((x) % water_grid_cols)])
                                * 32) / 129;
            end
        end

        for y = 1, water_grid_rows - 2 do
            for x = 1, water_grid_cols - 2 do
                -- compute average (downsample)
                local sum = water_grid[(y-1) * water_grid_cols + (x-1)] +
                            water_grid[(y-1) * water_grid_cols + (x)]   +
                            water_grid[(y-1) * water_grid_cols + (x+1)] +
                            water_grid[(y-1) * water_grid_cols + (x)]   +
                            water_grid[(y+1) * water_grid_cols + (x)]   +
                            water_grid[(y+1) * water_grid_cols + (x-1)] +
                            water_grid[(y+1) * water_grid_cols + (x)]   +
                            water_grid[(y+1) * water_grid_cols + (x+1)]
                
                local avg = sum / 8
                
                local idx = (x / 8) * water_grid_rows + (y / 8)
                color_map_water[idx] = color_palette[trunc(clamp(avg:len(), 0, 255))]

                -- should not happen, but be safe
                if color_map_water[idx] == nil then
                    color_map_water[idx] = color_palette[0]
                end
            end
        end
    end
end

function on_tick(delta)
    ticks = ticks + delta + 1

    local num_keys = get_num_keys()

    -- calculate waterfall effect
    compute_waterfall(ticks)

    -- calculate afterglow effect for pressed keys
    if ticks % afterglow_step == 0 then
        for i = 0, num_keys do        
            if color_map_pressed[i] >= 0x00000000 then
                color_map_pressed[i] = color_map_pressed[i] - color_step_afterglow

                if color_map_pressed[i] >= 0x00ffffff then
                    color_map_pressed[i] = 0x00ffffff
                elseif color_map_pressed[i] <= 0x00000000 then
                    color_map_pressed[i] = 0x00000000
                end
            end
        end
    end

    -- now combine all the color maps to a final map
    local color_map_combined = {}
    for i = 0, num_keys do
        color_map_combined[i] = color_map[i] + color_map_water[i] + color_map_pressed[i]

        -- let the afterglow effect override all other effects
        if color_map_pressed[i] > 0x00000000 then
            color_map_combined[i] = color_map_pressed[i]
        end

        if color_map_combined[i] >= 0x00ffffff then
            color_map_combined[i] = 0x00ffffff
        elseif color_map_combined[i] <= 0x00000000 then
            color_map_combined[i] = 0x00000000
        end
    end

    set_color_map(color_map_combined)

    -- debug_print_flow_field()
end

-- init global state
function init_state()
    local num_keys = get_num_keys()
    for i = 0, num_keys do
        color_map[i] = color_background
        color_map_pressed[i] = color_off
        color_map_water[i] = color_off
    end

    -- initialize water grid
    for y = 0, water_grid_rows do
        for x = 0, water_grid_cols do
            water_grid[x * water_grid_rows + y] = Vector.new(x, y)
        end
    end

     -- initialize palette
    for i = 0, 255 do
        color_palette[i] = hsl_to_color((i / 5) + 180, 1.0, min(0.5, ((i * 2) / 256)))
    end
end

-- ****************************************************************************
-- mebens' vector class
-- https://gist.github.com/mebens/1055480
-- ****************************************************************************
Vector = {}
Vector.__index = Vector

function Vector.__add(a, b)
  if type(a) == "number" then
    return Vector.new(b.x + a, b.y + a)
  elseif type(b) == "number" then
    return Vector.new(a.x + b, a.y + b)
  else
    return Vector.new(a.x + b.x, a.y + b.y)
  end
end

function Vector.__sub(a, b)
  if type(a) == "number" then
    return Vector.new(b.x - a, b.y - a)
  elseif type(b) == "number" then
    return Vector.new(a.x - b, a.y - b)
  else
    return Vector.new(a.x - b.x, a.y - b.y)
  end
end

function Vector.__mul(a, b)
  if type(a) == "number" then
    return Vector.new(b.x * a, b.y * a)
  elseif type(b) == "number" then
    return Vector.new(a.x * b, a.y * b)
  else
    return Vector.new(a.x * b.x, a.y * b.y)
  end
end

function Vector.__div(a, b)
  if type(a) == "number" then
    return Vector.new(b.x / a, b.y / a)
  elseif type(b) == "number" then
    return Vector.new(a.x / b, a.y / b)
  else
    return Vector.new(a.x / b.x, a.y / b.y)
  end
end

function Vector.__eq(a, b)
  return a.x == b.x and a.y == b.y
end

function Vector.__lt(a, b)
  return a.x < b.x or (a.x == b.x and a.y < b.y)
end

function Vector.__le(a, b)
  return a.x <= b.x and a.y <= b.y
end

function Vector.__tostring(a)
  return "(" .. a.x .. ", " .. a.y .. ")"
end

function Vector.new(x, y)
  return setmetatable({ x = x or 0, y = y or 0 }, Vector)
end

function Vector.distance(a, b)
  return (b - a):len()
end

function Vector:clone()
  return Vector.new(self.x, self.y)
end

function Vector:unpack()
  return self.x, self.y
end

function Vector:len()
  return math.sqrt(self.x * self.x + self.y * self.y)
end

function Vector:lenSq()
  return self.x * self.x + self.y * self.y
end

function Vector:normalize()
  local len = self:len()
  self.x = self.x / len
  self.y = self.y / len
  return self
end

function Vector:normalized()
  return self / self:len()
end

function Vector:rotate(phi)
  local c = math.cos(phi)
  local s = math.sin(phi)
  self.x = c * self.x - s * self.y
  self.y = s * self.x + c * self.y
  return self
end

function Vector:rotated(phi)
  return self:clone():rotate(phi)
end

function Vector:perpendicular()
  return Vector.new(-self.y, self.x)
end

function Vector:projectOn(other)
  return (self * other) * other / other:lenSq()
end

function Vector:cross(other)
  return self.x * other.y - self.y * other.x
end

setmetatable(Vector, { __call = function(_, ...) return Vector.new(...) end })


function debug_print_flow_field()
    info("*********************************************************************************************************")

    local row = ""

    for y = 0, water_grid_rows do
        for x = 0, water_grid_cols do
            local val = water_grid[x * water_grid_rows + y]
            if val == nil then val = "nil" end

            local str = val:unpack()
            row = row .. " | " .. str
        end

        info(row)
        row = ""
    end

    info("*********************************************************************************************************")
end