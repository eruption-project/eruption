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


from eruption.transport.local import LocalTransport


class Connection:
    """Connection to a running instance of the Eruption daemon"""

    # Connection types
    UNKNOWN: int = 0  # unknown connection
    LOCAL: int = 1  # local transport
    REMOTE: int = 2  # type REMOTE is currently not implemented

    def __init__(self, *args, **kwargs):
        if not ("type" in kwargs and kwargs["type"] == Connection.LOCAL):
            raise InvalidParam("Invalid or unsupported connection type")
        else:
            self.connection_type = kwargs["type"]

    def connect(self):
        """Connect to a running instance of Eruption"""
        self._con = LocalTransport()
        self._con.connect()

        self.connected = True

    def disconnect(self):
        """Disconnect from Eruption"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        self._con = None
        self.connected = False

    def is_connected(self):
        """Get connection state, returns True if we are connected to a running Eruption
           instance, otherwise returns False"""
        return self.connected

    def get_server_status(self):
        """Get the status of a running Eruption instance"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.get_server_status()
        return result

    def get_active_profile(self):
        """Get the file path of the active profile"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.get_active_profile()
        return result

    def switch_profile(self, profile_file):
        """Switches the active profile to one given in the file path"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.switch_profile(profile_file)
        return result

    def set_parameters(self, profile_file, script_file, **parameters):
        """Set parameters for the given profile's script"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.set_parameters(profile_file, script_file, **parameters)
        return result

    def submit_canvas(self, canvas, *args, **kwargs):
        """Submit the canvas to Eruption for realization"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.submit_canvas(canvas)
        return result

    def notify_device_hotplug(self, hotplug_info, *args, **kwargs):
        """Notify Eruption about a device hotplug event"""
        if not self.is_connected():
            raise NotConnectedError("Not connected")

        result = self._con.notify_device_hotplug(hotplug_info)
        return result


class InvalidParam(Exception):
    pass


class NotConnectedError(Exception):
    pass


class ConnectionFailed(Exception):
    pass
