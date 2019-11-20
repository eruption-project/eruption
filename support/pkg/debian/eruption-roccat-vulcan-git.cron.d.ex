#
# Regular cron jobs for the eruption-roccat-vulcan-git package
#
0 4	* * *	root	[ -x /usr/bin/eruption-roccat-vulcan-git_maintenance ] && /usr/bin/eruption-roccat-vulcan-git_maintenance
