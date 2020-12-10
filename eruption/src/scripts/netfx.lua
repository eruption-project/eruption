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
server = nil
conn = nil
initialized = false

-- utility functions --
local function split(str, pat)
    local t = {}
    local fpat = "(.-)" .. pat
    local last_end = 1
    local s, e, cap = str:find(fpat, 1)

    while s do
       if s ~= 1 or cap ~= "" then
          table.insert(t, cap)
       end
       last_end = e + 1
       s, e, cap = str:find(fpat, last_end)
    end

    if last_end <= #str then
       cap = str:sub(last_end)
       table.insert(t, cap)
    end

    return t
end

-- event handler functions --
function on_startup(config)
    for i = 0, canvas_size do
        color_map[i] = 0x00000000
    end

    -- bind server socket
    local status, socket = pcall(require, "socket")
    if not status then
        error("Your system is missing a required Lua library. You may want to install a package named like 'lua51-socket' or 'lua-socket-compat'")
    else
        server = socket.tcp()

        -- configure socket
        server:setoption("reuseaddr", true)
        -- server:setoption("reuseport", true)

        local status, msg = server:bind(bind_address, port)
        if status == nil then
            error("Network FX: Could not bind socket to the specified address: " .. msg)
            return
        end

        local status, msg = server:listen(0)
        if status == nil then
            error("Network FX: Could not transition socket to listening state: " .. msg)
            return
        end

        -- set non-blocking mode
        server:settimeout(0)

        local ip, port = server:getsockname()
        info("Network FX: Server now listening on " .. ip .. ":" .. port)

        initialized = true
    end
end

function on_quit()
    if server ~= nil then
        info("Network FX: Server shutting down")
        server:close()
    end
end

function on_tick(delta)
    if initialized then
        if conn == nil then
            -- we currently have no client connected, so poll for pending connection requests
            conn = server:accept()
            if conn ~= nil then
                -- a new client connected

                -- set non-blocking mode
                conn:settimeout(0)

                local ip, port = conn:getpeername()
                info("Network FX: Client connected from " .. ip .. ":" .. port)
            end
        else
            -- receive and process data
            local data, status

            repeat
                data, status = conn:receive()

                if data ~= nil then
                    trace("Network FX: Request: " .. data)

                    -- check for, and process protocol commands
                    if data == "QUIT" then
                        conn:send("BYE\n")
                        conn:close()
                        conn = nil
                        return
                    elseif data == "STATUS" then
                        conn:send("Eruption Network FX / Protocol version: 1.0\n")
                        return
                    end

                    -- data is apparently not a command
                    local result = split(data, ':')

                    -- validate request parameters
                    if #result ~= 5 then
                        error("Network FX: Request is ill-formed")

                        conn:send("ERROR: 100\n")
                        conn:close()
                        conn = nil
                        return
                    end

                    local r, g, b, a = tonumber(result[2]), tonumber(result[3]),
                                       tonumber(result[4]), tonumber(result[5])

                    -- validate colors
                    if r < 0 or r > 255 or g < 0 or g > 255 or
                       b < 0 or b > 255 or a < 0 or a > 255 then
                        error("Network FX: Color component value out of range")

                        conn:send("ERROR: 110\n")
                        conn:close()
                        conn = nil
                        return
                    end

                    local color = rgba_to_color(r, g, b, a)

                    local components = split(result[1], ',')

                    for idx = 1, #components do
                        local spec = split(components[idx], '-')
                        if #spec < 2 then
                            if result[1] == "ALL" then
                                -- predefined zone: full canvas
                                for i = 0, canvas_size do
                                    color_map[i] = color
                                end
                            else
                                -- set a single pixel on the canvas to a specific color
                                if components[idx] ~= nil and components[idx] ~= '' and
                                   tonumber(components[idx]) > 0 and
                                   tonumber(components[idx]) <= canvas_size then
                                    local index = tonumber(components[idx])
                                    color_map[index] = color
                                else
                                    error("Network FX: Invalid index")

                                    conn:send("ERROR: 120\n")
                                    conn:close()
                                    conn = nil
                                    return
                                end
                            end
                        else
                            -- set a range of pixels to a specific color
                            if spec[1] ~= nil and spec[2] ~= nil and
                               spec[1] ~= ''  and spec[2] ~= '' and
                               tonumber(spec[1]) > 0 and tonumber(spec[1]) <= canvas_size and
                               tonumber(spec[2]) > 0 and tonumber(spec[2]) <= canvas_size then
                                local low = tonumber(spec[1])
                                local high = tonumber(spec[2])

                                for index = low, high do
                                    color_map[index] = color
                                end
                            else
                                error("Network FX: Invalid index")

                                conn:send("ERROR: 120\n")
                                conn:close()
                                conn = nil
                                return
                            end
                        end

                        submit_color_map(color_map)
                    end

                    conn:send("OK\n")
                else
                    -- socket:read returned 'nil'

                    if status == "closed" then
                        -- lost connection to client
                        info("Network FX: Client disconnected")

                        conn = nil
                    elseif status == "timeout" then
                        -- don't do anything on timeouts
                    end
                end
            until data == nil
        end
    end
end
