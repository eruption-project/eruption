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
#
#    Copyright (c) 2019-2022, The Eruption Development Team

import sys
import tempfile
import textwrap
import time
import uuid

from eruption import SDK_VERSION, Connection, Canvas, Color

EXAMPLE_NAME = "Profile Control Python Example #1"


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

        original_profile = connection.get_active_profile()
        print("Original profile:", original_profile)

        # So that we don't interfere with a real profile, let's just create a new one
        with tempfile.NamedTemporaryFile(mode="w", suffix=".profile") as new_profile_file:
            solid_script = "/usr/share/eruption/scripts/solid.lua" # Solid background color
            wave_script = "/usr/share/eruption/scripts/wave.lua"   # Dark wave pattern

            new_profile_file.write(textwrap.dedent(f"""\
                id = '{uuid.uuid4()}'
                name = 'Profile Control Demonstration'
                description = 'Create and control a profile from Python'
                active_scripts = ['{solid_script}', '{wave_script}']
                """))
            new_profile_file.flush()

            print()
            print("Switching to new profile", new_profile_file.name, "using default script parameters.")
            switched = connection.switch_profile(new_profile_file.name)

            if switched:
                new_profile = connection.get_active_profile()
                print("Switched profile:", new_profile)
            else:
                print("Could not switch profiles")
                return

            time.sleep(3)

            print()
            print("Updating parameters #1 - Warp core mode")
            # Each script has its own parameters, and the script needs to be specified when setting its parameters.
            # The names and type of the parameters are defined by the script's manifest.
            connection.set_parameters(new_profile_file.name, solid_script, color_background='#ff10a0ff')
            connection.set_parameters(new_profile_file.name, wave_script, horizontal=True, direction=1, wave_length=2, speed_divisor=15)

            time.sleep(3)

            print("Updating parameters #2 - Warp factor 9")
            # Only the parameters that change need to be specified.
            connection.set_parameters(new_profile_file.name, wave_script, wave_length=1, speed_divisor=1.5)

            time.sleep(3)

            print("Updating parameters #3 - Too fast, reverse course")
            # Setting the parameters using a dictionary.
            parameters = {
                'direction': -1,
                'wave_length': 2,
                'speed_divisor': 25
            }
            connection.set_parameters(new_profile_file.name, wave_script, **parameters)

            time.sleep(3)

        print()
        print("Reverting to original profile")
        connection.switch_profile(original_profile)

        connection.disconnect()
        print("Exiting now")

    except (Exception) as e:
        print(f"An error occurred: {type(e).__name__} {e}")


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
