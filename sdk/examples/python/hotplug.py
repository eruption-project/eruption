#!/bin/env python3
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

import sys
import time

from eruption import SDK_VERSION, Connection, Canvas, Color
from eruption.hardware import HotplugInfo

EXAMPLE_NAME = "Python Example 'hotplug'"


def main():
    """Main program entrypoint"""

    print(f"Welcome to the Eruption SDK!\nYou are running the \"{EXAMPLE_NAME}\" "
          f"from the Eruption SDK version {SDK_VERSION}\n")

    # connect to the Eruption daemon (via a local connection)
    try:
        print("Connecting to the Eruption daemon...")
        connection = Connection(type=Connection.LOCAL)

        connection.connect()
        print("Successfully connected to the Eruption daemon")

        status = connection.get_server_status()
        print(status)

        hotplug_info = HotplugInfo()
        connection.notify_device_hotplug(hotplug_info)

        connection.disconnect()
        print("Exiting now")

    except (Exception) as e:
        print(f"An error occurred: {type(e).__name__} {e}")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
