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


import socket
import google.protobuf
import google.protobuf.internal.encoder as GoogleProtobufEncoder

from eruption.transport.sdk_support_pb2 import Request, Response, RequestType

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
        request = Request(request_type=RequestType.STATUS)

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return { "server": str(response.data, 'utf-8') }

    def submit_canvas(self, canvas):
        """Submit the canvas to Eruption for realization"""
        request = Request(request_type=RequestType.SET_CANVAS)

        color_bytes = []
        for color in canvas.data:
            color_bytes.append(color.data[0])
            color_bytes.append(color.data[1])
            color_bytes.append(color.data[2])
            color_bytes.append(color.data[3])

        request.data = bytes(color_bytes)

        buf = GoogleProtobufEncoder._VarintBytes(request.ByteSize())
        buf += request.SerializeToString()
        cnt = self.socket.send(bytes(buf))

        recv_buf = self.socket.recv(MAX_BUF)
        response = Response()
        response.ParseFromString(recv_buf[1:])

        return response


class RequestFailed(Exception):
    pass
