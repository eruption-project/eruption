#    SPDX-License-Identifier: GPL-2.0
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
#    Copyright (c) 2019-2023, The Eruption Development Team


KDIR ?= /lib/modules/`uname -r`/build

default:
	$(MAKE) -C $(KDIR) M=$$PWD/src

clean:
	rm -f src/modules.order src/Module.symvers src/eruption_blackout.o src/eruption_blackout.mod.o \
		  src/eruption_blackout.mod.c src/eruption_blackout.mod \
		  src/.eruption_blackout.ko.cmd src/.eruption_blackout.mod.cmd \
		  src/.eruption_blackout.mod.o.cmd src/.eruption_blackout.o.cmd \
		  src/roccat_vulcan_tkl_pro.o src/mod.o \
		  src/hwdevices/mod.mod src/hwdevices/roccat_vulcan_tkl_pro.mod \
		  src/hwdevices/.mod.mod.cmd src/hwdevices/.mod.o.cmd \
		  src/hwdevices/.roccat_vulcan_tkl_pro.mod.cmd src/hwdevices/.roccat_vulcan_tkl_pro.o.cmd \
		  src/.modules.order.cmd src/.Module.symvers.cmd \
		  src/eruption_blackout.ko
