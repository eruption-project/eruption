#!/bin/bash
#  SPDX-License-Identifier: GPL-3.0-or-later
#
#  This file is part of Eruption.
#
#  Eruption is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  Eruption is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
#
#  Copyright (c) 2019-2023, The Eruption Development Team

function gen_manpages {
	./target/release/"$1" "manpages"
}

# supported locales
languages=('en_US')

for l in "${languages[@]}"; do
	export LANG=$l
	export MANPAGES_OUTPUT_DIR="support/shell/manpages/$LANG/"
	mkdir -p "support/shell/manpages/$LANG/"

	# gen_manpages "eruption"
	# gen_manpages "eruption-cmd"
	# gen_manpages "eruption-hwutil"
	# gen_manpages "eruption-debug-tool"
	# gen_manpages "eruption-gui-gtk3"
	# gen_manpages "eruption-macro"
	# gen_manpages "eruption-keymap"
	# gen_manpages "eruption-netfx"
	# gen_manpages "eruption-fx-proxy"
	# gen_manpages "eruption-audio-proxy"
	# gen_manpages "eruption-process-monitor"
	gen_manpages "eruptionctl"
	# gen_manpages "pyroclasm"
done

exit 0
