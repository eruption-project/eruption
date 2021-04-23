#!/bin/sh

if [ "$1" = "pre" ] ; then
    # prepare Eruption for system sleep
    touch /run/lock/eruption-hotplug-helper.lock

    systemctl stop eruption-hotplug-helper.service
	systemctl stop eruption.service

    touch /run/lock/eruption-sleep.lock
else
    # wake up Eruption after system sleep
    rm /run/lock/eruption-hotplug-helper.lock
    systemctl start eruption-hotplug-helper.service
    rm /run/lock/eruption-sleep.lock
fi
