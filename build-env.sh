#!/bin/sh
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

NIGHTLY_FEATURES='cargo-features = \["codegen-backend", "profile-rustflags"\]'
STABLE_FEATURES='# cargo-features = \["codegen-backend", "profile-rustflags"\]'

switch_to_nightly() {
    ln -fs config-cranelift.toml .cargo/config.toml

    sed -i~ -e "s/^$STABLE_FEATURES$/$NIGHTLY_FEATURES/" Cargo.toml
}

switch_to_stable() {
    ln -fs config-default.toml .cargo/config.toml

    sed -i~ -e "s/^$NIGHTLY_FEATURES$/$STABLE_FEATURES/" Cargo.toml
}

case "$1" in
    "default" | "stable")
        switch_to_stable;
        echo "Switched to the 'default' build environment"
        ;;

    "cranelift" | "nightly")
        switch_to_nightly;
        echo "Enabled 'cranelift' build environment (using rust nightly)"
        ;;

    "help" | "--help")
        echo "Specify one of 'default' or 'cranelift'"
        ;;

    *)
        echo "Invalid argument '$1'. Please specify one of 'default' or 'cranelift'"
        ;;
esac
