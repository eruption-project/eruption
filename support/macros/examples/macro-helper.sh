#!/bin/bash
#    SPDX-License-Identifier: GPL-3.0-or-later
#
#    This file is part of Eruption.
#
#    Eruption is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    Eruption is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
#
#    Copyright (c) 2019-2022, The Eruption Development Team


# Please use this file as a reference for running shell scripts
# from Lua macros, as your own Linux user instead of root
#
# Invoke it from a Lua script like this:
#
# result = system("/usr/bin/sudo", {"-u", "user", "/home/user/.local/bin/macro-helper.sh"})
# if result ~= 0 then
#     error("Command execution failed with result: " .. result)
# end

USER=`whoami`
USER_ID=`id -u`
DISPLAY=":1"

DBUS_SESSION_BUS_ADDRESS=unix:path=/run/$USER/$USER_ID/bus

export DISPLAY=$DISPLAY
export DBUS_SESSION_BUS_ADDRESS=$DBUS_SESSION_BUS_ADDRESS

/usr/bin/notify-send "Message from Eruption"
