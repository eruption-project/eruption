#!/bin/env python3

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

import sys
import time

from eruption import SDK_VERSION, Connection, Canvas, Color

EXAMPLE_NAME = "Simple Python Example #1"

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

        # create a new canvas
        canvas = Canvas()

        red = Color(255, 0, 0, 128)
        green = Color(0, 255, 0, 128)
        blue = Color(0, 0, 255, 128)
        final = Color(0, 0, 0, 0)

        canvas.fill(red)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(green)

        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(blue)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(final)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        connection.disconnect()
        print("Exiting now")

    except (Exception) as e:
        print(f"An error occurred: {type(e).__name__} {e}" )


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
