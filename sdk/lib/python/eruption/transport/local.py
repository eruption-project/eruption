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


import socket
import google.protobuf
import google.protobuf.internal.encoder as GoogleProtobufEncoder

from eruption.transport.sdk_support_pb2 import Request, Response

SOCKET_ADDRESS = "/run/eruption/control.sock"
MAX_BUF = 4096


class LocalTransport:
    """The Local transport (connects to Eruption via a UNIX domain socket)"""

    def __init__(self):
        """Connect to the Eruption daemon via a UNIX domain socket"""
        self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_SEQPACKET, 0)

    def connect(self):
        """Connect to a local instance of Eruption via a UNIX domain socket"""
        address = SOCKET_ADDRESS
        self.socket.connect(address)

    def disconnect(self):
        """Disconnect from Eruption"""
        if self.socket != None:
            self.socket.close()
            self.socket = None

    def get_server_status(self):
        """Get status of a running Eruption instance"""
        request = Request()
        request.status.SetInParent()

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return {"server": response.status.description}

    def get_active_profile(self):
        """Get the file path of the active profile"""
        request = Request()
        request.active_profile.SetInParent()

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response.active_profile.profile_file

    def switch_profile(self, profile_file):
        """Switches the active profile to one given in the file path"""
        request = Request()
        request.switch_profile.profile_file = profile_file

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response.switch_profile.switched

    def set_parameters(self, profile_file, script_file, **parameters):
        """Update parameter values for the given profile and script"""

        request = Request()
        request.set_parameters.profile_file = profile_file
        request.set_parameters.script_file = script_file
        for name, value in parameters.items():
            request.set_parameters.parameter_values[name] = str(value)

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response

    def submit_canvas(self, canvas):
        """Submit the canvas to Eruption for realization"""

        color_bytes = []
        for color in canvas.data:
            color_bytes.append(color.data[0])
            color_bytes.append(color.data[1])
            color_bytes.append(color.data[2])
            color_bytes.append(color.data[3])

        request = Request()
        request.set_canvas.canvas = bytes(color_bytes)

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response

    def notify_device_hotplug(self, hotplug_info):
        """Notify Eruption about a device hotplug event"""
        
        data_bytes = []
        data_bytes.append(hotplug_info.usb_vid)
        data_bytes.append(hotplug_info.usb_pid)

        request = Request()
        request.hotplug_request.payload = bytes(data_bytes)

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response


class RequestFailed(Exception):
    pass
