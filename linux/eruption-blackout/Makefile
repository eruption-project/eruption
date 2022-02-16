# SPDX-License-Identifier: GPL-2.0

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
