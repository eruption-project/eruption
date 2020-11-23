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
require "utilities"
require "debug"

-- global state variables --
STATE_INVALID = -1
STATE_INITIALIZING = 0
STATE_RUNNING = 1
STATE_GAME_OVER = 2

stage_state = STATE_INVALID
frame_counter = 0

color_map = {}
food_map = {}
ticks = 0

current_direction = 1
snake_list = {}

next_change_ticks = 1

delta_x = -1
delta_y = 0

max_food_ttl = 128

-- event handler functions --
function on_startup(config)
	reset_state()
end

function on_key_down(key_index)
	food_map[key_index] = max_food_ttl
end

function on_tick(delta)
	ticks = ticks + delta

	if stage_state == STATE_INITIALIZING then
		step_and_draw_init(ticks)
	elseif stage_state == STATE_GAME_OVER then
		step_and_draw_game_over(ticks)
	elseif stage_state == STATE_RUNNING then
		step_and_draw_game(ticks)
	else
		error("Invalid internal state")
	end
end

function step_and_draw_init(ticks)
	for i = 0, num_keys do
		local val = max(64 - (frame_counter * 4), 0)
		color_map[i] = rgba_to_color(val, val, val, val)
	end

	submit_color_map(color_map)

	if frame_counter <= 0 then
		stage_state = STATE_RUNNING
	end

	frame_counter = frame_counter - 1
end

function step_and_draw_game_over(ticks)
	-- if frame_counter > 32 delay

	if frame_counter <= 32 then
		for i = 0, num_keys do
			color_map[i] = 0x00000000
		end

		submit_color_map(color_map)
	end

	if frame_counter <= 0 then
		reset_state()
	end

	frame_counter = frame_counter - 1
end

function step_and_draw_game(ticks)
	if ticks % 8 == 0 then
		-- advance snake body by one block
		local x = snake_list[1]
		local y = snake_list[2]

		local new_x = x + delta_x
		local new_y = y + delta_y

		-- if new_x <= 1 then
		-- 	if new_y <= max_keys_per_col / 2 then
		-- 		change_direction(4)
		-- 	else
		-- 		change_direction(2)
		-- 	end
		-- end

		-- if new_y <= 1 then
		-- 	if new_x <= max_keys_per_row / 2 then
		-- 		change_direction(1)
		-- 	else
		-- 		change_direction(3)
		-- 	end
		-- end

		-- if new_x >= max_keys_per_row - 1 then
		-- 	if new_y <= max_keys_per_col / 2 then
		-- 		change_direction(4)
		-- 	else
		-- 		change_direction(2)
		-- 	end
		-- end

		-- if new_y >= max_keys_per_col - 1 then
		-- 	if new_x >= max_keys_per_row / 2 then
		-- 		change_direction(1)
		-- 	else
		-- 		change_direction(3)
		-- 	end
		-- end

		-- assertions
		if new_x < 1 then new_x = max_keys_per_row end
		if new_y < 1 then new_y = max_keys_per_col end

		if new_x > max_keys_per_row then new_x = 1 end
		if new_y > max_keys_per_col then new_y = 1 end

		if x > max_keys_per_row then change_direction(1) end
		if y > max_keys_per_col then change_direction(2) end

		if x < 0 then change_direction(3) end
		if y < 0 then change_direction(4) end

		-- debug("Snake: current_direction: " .. current_direction ..
		-- 	  " old x: " .. x .. " old y: " .. y ..
		-- 	  " new x: " .. new_x .. " new y: " .. new_y)

		-- check for self-collision
		-- for j = 0, #snake_list, 2 do
		-- 	local x = snake_list[j + 1]
		-- 	local y = snake_list[j + 2]

		-- 	if x ~= nil and y ~= nil and
		-- 	   x == new_x and y == new_y then
		-- 		game_over()
		-- 	end
		-- end

		-- collision detection: snake vs. food
		local idx = key_index(new_x, new_y)
		if food_map[idx] ~= nil and food_map[idx] > 0 then
			debug("Snake: Collected some food!")

			food_map[idx] = 0

			-- eat food and grow (add tail segment)
			local tmp_x = snake_list[#snake_list - 1]
			local tmp_y = snake_list[#snake_list - 2]

			table.insert(snake_list, #snake_list, tmp_y)
			table.insert(snake_list, #snake_list, tmp_x)

			-- change direction
			randomize_direction()
		end

		table.insert(snake_list, 1, new_y)
		table.insert(snake_list, 1, new_x)

		-- remove last segment
		table.remove(snake_list, #snake_list)
		table.remove(snake_list, #snake_list)
	end

	-- if ticks % next_change_ticks == 0 then
	--	randomize_direction()
	-- end

	-- clear background, draw food and manage state
	for i = 0, num_keys do
		if food_map[i] > 0 then
			color_map[i] = rgba_to_color(255, 0, 0, 255)
		else
			color_map[i] = 0x00000000
		end

		food_map[i] = food_map[i] - 1
	end

	-- draw the snake
	for j = 0, #snake_list, 2 do
		local x = snake_list[j + 1]
		local y = snake_list[j + 2]

		if x ~= nil and y ~= nil then
			local idx = key_index(x, y)
			if idx == nil then idx = 1 else idx = idx + 1 end

			-- trace("x: " .. x .. " y: " .. y .. " idx: " .. idx)

			if x ~= nil and y ~= nil then
				color_map[idx] = rgba_to_color(255, 40, 0, 255)
			end
		end
	end

	submit_color_map(color_map)
end

function randomize_direction()
	if current_direction == 1 or current_direction == 3 then
		if abs(rand(0, 100)) < 50 then
			change_direction(2)
		else
			change_direction(4)
		end

		next_change_ticks = abs(rand(50, 200))
	else
		if abs(rand(0, 100)) < 50 then
			change_direction(1)
		else
			change_direction(3)
		end

		next_change_ticks = abs(rand(250, 400))
	end
end

function change_direction(dir)
	if dir == 1 then
		debug("Snake: Changing direction: left")
		current_direction = 1

		delta_x = -1
		delta_y = 0

	elseif dir == 2 then
		debug("Snake: Changing direction: up")
		current_direction = 2

		delta_x = 0
		delta_y = -1

	elseif dir == 3 then
		debug("Snake: Changing direction: right")
		current_direction = 3

		delta_x = 1
		delta_y = 0

	elseif dir == 4 then
		debug("Snake: Changing direction: down")
		current_direction = 4

		delta_x = 0
		delta_y = 1

	else
		error("Snake: Invalid direction: " .. dir)
	end
end

function game_over()
	debug("Snake: Game over!")

	frame_counter = 64
	stage_state = STATE_GAME_OVER
end

function reset_state()
    for i = 0, num_keys do
		color_map[i] = 0x00000000
		food_map[i] = 0
	end

	change_direction(3)

	snake_list = {
		5, 3,
		4, 3,
		3, 3,
	}

	stage_state = STATE_INITIALIZING
	frame_counter = 32
end
