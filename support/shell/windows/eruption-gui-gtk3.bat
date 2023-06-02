@REM  SPDX-License-Identifier: GPL-3.0-or-later
@REM
@REM  This file is part of Eruption.
@REM
@REM  Eruption is free software: you can redistribute it and/or modify
@REM  it under the terms of the GNU General Public License as published by
@REM  the Free Software Foundation, either version 3 of the License, or
@REM  (at your option) any later version.
@REM
@REM  Eruption is distributed in the hope that it will be useful,
@REM  but WITHOUT ANY WARRANTY; without even the implied warranty of
@REM  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
@REM  GNU General Public License for more details.
@REM
@REM  You should have received a copy of the GNU General Public License
@REM  along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
@REM
@REM  Copyright (c) 2019-2023, The Eruption Development Team


:: This batch-script initializes and runs the Eruption GUI on Microsoft Windows

title "Eruption GTK3+ GUI"

set /p DBUS_SYSTEM_BUS_ADDRESS=<%AppData%\eruption\dbus-system-bus.txt
set /p DBUS_SESSION_BUS_ADDRESS=<%AppData%\eruption\dbus-session-bus.txt

set RUST_BACKTRACE=full
set RUST_LOG=info

eruption-gui-gtk3.exe -c .\etc\eruption.conf
