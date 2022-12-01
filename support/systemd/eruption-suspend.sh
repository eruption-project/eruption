#!/bin/sh
#    SPDX-License-Identifier: GPL-3.0-or-later
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


if [ "$1" = "pre" ] ; then
    # prepare Eruption for system sleep

    touch /run/lock/eruption-hotplug-helper.lock

    systemctl stop eruption-hotplug-helper.service
    systemctl stop eruption.service

    touch /run/lock/eruption-sleep.lock
else
    # wake up Eruption after system sleep

    systemctl reset-failed eruption-hotplug-helper.service
    systemctl reset-failed eruption.service

    rm /run/lock/eruption-hotplug-helper.lock

    systemctl start eruption-hotplug-helper.service

    rm /run/lock/eruption-sleep.lock
fi
