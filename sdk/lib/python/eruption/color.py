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


class Color:
    """A RGB(A) color value"""

    def __init__(self, *args):
        """Create a new RGB(A) color value"""
        self.data = args

    def r(self):
        """Returns the RED channel of the color"""
        self.data[0]

    def g(self):
        """Returns the GREEN channel of the color"""
        self.data[1]

    def b(self):
        """Returns the BLUE channel of the color"""
        self.data[2]

    def a(self):
        """Returns the ALPHA channel of the color"""
        self.data[3]

    def set_r(self, val):
        """Sets the RED channel of the color"""
        self.data[0] = val

    def set_g(self, val):
        """Sets the GREEN channel of the color"""
        self.data[1] = val

    def set_b(self, val):
        """Sets the BLUE channel of the color"""
        self.data[2] = val

    def set_a(self, val):
        """Sets the ALPHA channel of the color"""
        self.data[3] = val
