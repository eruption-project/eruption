#!/bin/sh

if [ "$1" = "pre" ] ; then
    # stop eruption.service when entering suspend or hibernate
	systemctl stop eruption.service
else
    # start eruption.service when waking up from suspend or hibernate
	systemctl start eruption.service
fi
