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

-- utility functions --

-- load the state table from the ephemeral store
local function load_state_table()
	local len = load_int_transient("profiles.saved_state_table.len", 0)
	if len == 0 then return {} end

	local result = {}
	for i = 1, len do
		local key = "profiles.saved_state_table[" .. i .. "]"

		local pid = load_int_transient(key .. ".pid", -1)
		local type = load_string_transient(key .. ".type", "number")

		local mapping
		if type == "string" then
			mapping = load_string_transient(key .. ".profile", "default.profile")
		elseif type == "number" then
			mapping = load_int_transient(key .. ".slot", 1)
		else
			error("Unhandled type in 'load_state_table'")
		end

		local elem = { pid, mapping }
		table.insert(result, elem)
	end

	return result
end

-- store the state table to the ephemeral store
local function store_state_table()
	if SAVED_STATE_TABLE == nil or #SAVED_STATE_TABLE == nil or
	  #SAVED_STATE_TABLE == 0 then
		return
	end

	local len = #SAVED_STATE_TABLE
	store_int_transient("profiles.saved_state_table.len", len)

	for i = 1, len do
		local key = "profiles.saved_state_table[" .. i .. "]"

		local pid = SAVED_STATE_TABLE[i][1]
		local mapping = SAVED_STATE_TABLE[i][2]

		store_int_transient(key .. ".pid", pid)

		if type(mapping) == "string" then
			store_string_transient(key .. ".type", "string")
			store_string_transient(key .. ".profile", mapping)
		elseif type(mapping) == "number" then
			store_string_transient(key .. ".type", "number")
			store_int_transient(key .. ".slot", mapping)
		else
			error("Unhandled type in 'store_state_table'")
		end
	end
end

PROCESS_MAPPING_TABLE = {}  -- stores `file name` to `slot/profile` mappings
SAVED_STATE_TABLE = load_state_table()  -- stores `pid` to `previous state` mappings

local function do_switch_to_slot(index)
	switch_to_slot(index)
end

local function do_switch_to_profile(profile_name)
	switch_to_profile(profile_name)
end

local function find_in_table(t, needle)
	for k, v in pairs(t) do
		local val = v[1]
		local result = v[2]

		if val == needle then
			return result
		end
	end

	return nil
end

local function save_current_state(pid, file_name, mapping)
	if type(mapping) == "string" then
		debug("Profiles: Saving state: [" .. pid .. "] " .. file_name .. " => " .. mapping)
	elseif type(mapping) == "number" then
		debug("Profiles: Saving state: [" .. pid .. "] " .. file_name .. " => slot #" .. mapping)
	else
		error("Profiles: Unhandled type in 'save_current_state'")
	end

	table.insert(SAVED_STATE_TABLE, { pid, mapping })

	store_state_table()
end

-- library functions --
function process_mapping(file_name, mapping)
	if type(mapping) == "string" then
		debug("Profiles: Adding mapping: " .. file_name .. " => switch to profile: " .. mapping)
		table.insert(PROCESS_MAPPING_TABLE, { file_name, mapping })
	elseif type(mapping) == "number" then
		debug("Profiles: Adding mapping: " .. file_name .. " => switch to slot #" .. mapping)
		table.insert(PROCESS_MAPPING_TABLE, { file_name, mapping })
	else
		error("Profiles: Unhandled type in 'process_mapping'")
	end
end

-- event handler functions --
function on_process_exec(pid, file_name, hash)
	trace("Profiles: Process execution: [" .. pid .. "]: " .. file_name)

	local mapping = find_in_table(PROCESS_MAPPING_TABLE, file_name)
	if mapping ~= nil then
		if type(mapping) == "string" then
			info("Profiles: Performing action for: " .. file_name .. " switching to profile: " ..
				 mapping .. ", on the current slot")

			save_current_state(pid, file_name, get_current_profile())
			do_switch_to_profile(mapping)
		elseif type(mapping) == "number" then
			info("Profiles: Performing action for: " .. file_name .. ", switching to slot #" .. mapping)

			save_current_state(pid, file_name, get_current_slot() + 1)
			do_switch_to_slot(mapping - 1)
		else
			error("Profiles: Unhandled type in 'on_process_exec'")
		end
	end
end

function on_process_exit(pid, file_name, hash)
	trace("Profiles: Process terminated: [" .. pid .. "]: " .. file_name)

	local previous_state = find_in_table(SAVED_STATE_TABLE, pid)
	if previous_state ~= nil then
		if type(previous_state) == "string" then
			info("Profiles: Restoring previous state: [" .. pid .. "] terminated => switching to profile: " .. previous_state)
			do_switch_to_profile(previous_state)
		elseif type(previous_state) == "number" then
			info("Profiles: Restoring previous state: [" .. pid .. "] terminated => switching to slot #" .. previous_state)
			do_switch_to_slot(previous_state - 1)
		else
			error("Profiles: Unhandled type in 'on_process_exit'")
		end
	end
end

function on_system_event(code, arg1, arg2, arg3)
	trace("Profiles: System event: " .. code .. " arg1: " .. arg1 ..
		  " arg2: " .. arg2 .. " arg3: " .. arg3)

	if code == 0 then
		-- process exec() event
		local pid = arg1
		local file_name = arg2
		local hash = arg3

		on_process_exec(pid, file_name, hash)
	elseif code == 1 then
		-- process exit event
		local pid = arg1
		local file_name = arg2
		local hash = arg3

		on_process_exit(pid, file_name, hash)
	end
end

-- import custom mapping definitions sub-modules
require(requires)
