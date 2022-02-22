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


BUILDFLAGS := --all --release --features=sourceview
# BUILDFLAGS := --all --release

TARGET_DIR := /usr
SOURCE_DIR := target/release

all: build

build:
	@cargo build $(BUILDFLAGS)

install:
	@echo "Shutting down daemons from previous installations..."

	-@systemctl --global disable --now eruption-audio-proxy.service
	-@systemctl --global disable --now eruption-process-monitor.service

	-@systemctl mask eruption.service
	-@systemctl disable --now eruption.service

	@echo "Commencing installation of Eruption..."

	@mkdir -p "/etc/eruption"
	@mkdir -p "/usr/share/doc/eruption"
	@mkdir -p /usr/share/eruption/scripts/{lib/{macros,themes,hwdevices/{keyboards,mice}},examples}
	@mkdir -p "/usr/share/applications"
	@mkdir -p "/usr/share/icons/hicolor/64x64/apps"
	@mkdir -p "/usr/share/eruption-gui/schemas"
	@mkdir -p "/var/lib/eruption/profiles"
	@mkdir -p "/usr/lib/systemd/system"
	@mkdir -p "/usr/lib/systemd/system-preset"
	@mkdir -p "/usr/lib/systemd/user"
	@mkdir -p "/usr/lib/systemd/user-preset"
	@mkdir -p "/usr/lib/systemd/system-sleep"
	@mkdir -p "/usr/lib/udev/rules.d/"
	@mkdir -p "/usr/share/dbus-1/system.d"
	@mkdir -p "/usr/share/dbus-1/session.d"
	@mkdir -p "/usr/share/polkit-1/actions"
	@mkdir -p "/usr/share/man/man8"
	@mkdir -p "/usr/share/man/man5"
	@mkdir -p "/usr/share/man/man1"
	@mkdir -p "/usr/share/bash-completion/completions"
	@mkdir -p "/usr/share/fish/completions"
	@mkdir -p "/usr/share/zsh/site-functions"
	@mkdir -p "/usr/share/eruption/i18n"
	@mkdir -p "/usr/share/eruption/sfx"

	@cp "support/assets/eruption-gui/eruption-gui.desktop" "/usr/share/applications/"
	@cp "support/assets/eruption-gui/eruption-gui.png" "/usr/share/icons/hicolor/64x64/apps/"
	@cp "eruption-gui/schemas/gschemas.compiled" "/usr/share/eruption-gui/schemas/"
	@cp "support/systemd/eruption-suspend.sh" "/usr/lib/systemd/system-sleep/eruption"
	@cp "support/config/eruption.conf" "/etc/eruption/"
	@cp "support/config/audio-proxy.conf" "/etc/eruption/"
	@cp "support/config/process-monitor.conf" "/etc/eruption/"
	@cp "support/systemd/eruption.service" "/usr/lib/systemd/system/"
	@cp "support/systemd/eruption.preset" "/usr/lib/systemd/system-preset/50-eruption.preset"
	@cp "support/systemd/eruption-audio-proxy.service" "/usr/lib/systemd/user/"
	@cp "support/systemd/eruption-audio-proxy.preset" "/usr/lib/systemd/user-preset/50-eruption-audio-proxy.preset"
	@cp "support/systemd/eruption-process-monitor.service" "/usr/lib/systemd/user/"
	@cp "support/systemd/eruption-process-monitor.preset" "/usr/lib/systemd/user-preset/50-eruption-process-monitor.preset"
	@cp "support/systemd/eruption-hotplug-helper.service" "/usr/lib/systemd/system/"
	@cp "support/systemd/eruption-hotplug-helper.preset" "/usr/lib/systemd/system-preset/50-eruption-hotplug-helper.preset"
	@cp "support/udev/99-eruption.rules" "/usr/lib/udev/rules.d/"
	@cp "support/dbus/org.eruption.control.conf" "/usr/share/dbus-1/system.d/"
	@cp "support/dbus/org.eruption.process_monitor.conf" "/usr/share/dbus-1/session.d/"
	@cp "support/policykit/org.eruption.policy" "/usr/share/polkit-1/actions/"
	@cp "support/man/eruption.8" "/usr/share/man/man8/"
	@cp "support/man/eruption-cmd.8" "/usr/share/man/man8/"
	@cp "support/man/eruption.conf.5" "/usr/share/man/man5/"
	@cp "support/man/process-monitor.conf.5" "/usr/share/man/man5/"
	@cp "support/man/eruptionctl.1" "/usr/share/man/man1/"
	@cp "support/man/eruption-hwutil.8" "/usr/share/man/man8/"
	@cp "support/man/eruption-netfx.1" "/usr/share/man/man1/"
	@cp "support/man/eruption-audio-proxy.1" "/usr/share/man/man1/"
	@cp "support/man/eruption-process-monitor.1" "/usr/share/man/man1/"
	@cp "support/shell/completions/en_US/eruption-cmd.bash-completion" "/usr/share/bash-completion/completions/eruption-cmd"
	@cp "support/shell/completions/en_US/eruption-hwutil.bash-completion" "/usr/share/bash-completion/completions/eruption-hwutil"
	@cp "support/shell/completions/en_US/eruption-debug-tool.bash-completion" "/usr/share/bash-completion/completions/eruption-debug-tool"
	@cp "support/shell/completions/en_US/eruption-netfx.bash-completion" "/usr/share/bash-completion/completions/eruption-netfx"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.bash-completion" "/usr/share/bash-completion/completions/eruption-audio-proxy"
	@cp "support/shell/completions/en_US/eruption-process-monitor.bash-completion" "/usr/share/bash-completion/completions/eruption-process-monitor"
	@cp "support/shell/completions/en_US/eruptionctl.bash-completion" "/usr/share/bash-completion/completions/eruptionctl"
	@cp "support/shell/completions/en_US/eruption-cmd.fish-completion" "/usr/share/fish/completions/eruption-cmd.fish"
	@cp "support/shell/completions/en_US/eruption-hwutil.fish-completion" "/usr/share/fish/completions/eruption-hwutil.fish"
	@cp "support/shell/completions/en_US/eruption-debug-tool.fish-completion" "/usr/share/fish/completions/eruption-debug-tool.fish"
	@cp "support/shell/completions/en_US/eruption-netfx.fish-completion" "/usr/share/fish/completions/eruption-netfx.fish"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.fish-completion" "/usr/share/fish/completions/eruption-audio-proxy.fish"
	@cp "support/shell/completions/en_US/eruption-process-monitor.fish-completion" "/usr/share/fish/completions/eruption-process-monitor.fish"
	@cp "support/shell/completions/en_US/eruptionctl.fish-completion" "/usr/share/fish/completions/eruptionctl.fish"
	@cp "support/shell/completions/en_US/eruption-cmd.zsh-completion" "/usr/share/zsh/site-functions/_eruption-cmd"
	@cp "support/shell/completions/en_US/eruption-hwutil.zsh-completion" "/usr/share/zsh/site-functions/_eruption-hwutil"
	@cp "support/shell/completions/en_US/eruption-debug-tool.zsh-completion" "/usr/share/zsh/site-functions/_eruption-debug-tool"
	@cp "support/shell/completions/en_US/eruption-netfx.zsh-completion" "/usr/share/zsh/site-functions/_eruption-netfx"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.zsh-completion" "/usr/share/zsh/site-functions/_eruption-audio-proxy"
	@cp "support/shell/completions/en_US/eruption-process-monitor.zsh-completion" "/usr/share/zsh/site-functions/_eruption-process-monitor"
	@cp "support/shell/completions/en_US/eruptionctl.zsh-completion" "/usr/share/zsh/site-functions/_eruptionctl"
	@cp "support/sfx/typewriter1.wav" "/usr/share/eruption/sfx/"
	@cp "support/sfx/phaser1.wav" "/usr/share/eruption/sfx/"
	@cp "support/sfx/phaser2.wav" "/usr/share/eruption/sfx/"

	@chmod 0755 /usr/lib/systemd/system-sleep/eruption

	@ln -fs "phaser1.wav" "/usr/share/eruption/sfx/key-down.wav"
	@ln -fs "phaser2.wav" "/usr/share/eruption/sfx/key-up.wav"

	@cp -r eruption/src/scripts/* /usr/share/eruption/scripts/
	@cp -r support/profiles/* /var/lib/eruption/profiles/

	@cp target/release/eruption{,ctl,-cmd,-hwutil,-netfx,-debug-tool,-hotplug-helper,-gui,-audio-proxy,-process-monitor} /usr/bin/
	@setcap CAP_NET_ADMIN+ep /usr/bin/eruption-process-monitor

	@echo "Starting Eruption daemons..."

	-@systemctl daemon-reload
	-@systemctl --global daemon-reload

	-@systemctl reload systemd-udevd

	-@systemctl --global enable --now eruption-audio-proxy.service
	-@systemctl --global enable --now eruption-process-monitor.service

	-@systemctl unmask eruption.service
	-@systemctl enable --now eruption.service

	@echo ""
	@echo "Successfully installed Eruption"
	@echo ""

uninstall:
	@echo "Commencing removal of Eruption..."

	-@rm $(TARGET_DIR)/bin/eruption{,ctl,-cmd,-hwutil,-netfx,-debug-tool,-hotplug-helper,-gui,-audio-proxy,-process-monitor}

	-@rm /usr/share/applications/eruption-gui.desktop
	-@rm /usr/share/icons/hicolor/64x64/apps/eruption-gui.png
	-@rm /usr/share/eruption-gui/schemas/gschemas.compiled
	-@rm /usr/lib/systemd/system-sleep/eruption
	-@rm /usr/lib/systemd/system/eruption.service
	-@rm /usr/lib/systemd/system-preset/50-eruption.preset
	-@rm /usr/lib/systemd/user/eruption-audio-proxy.service
	-@rm /usr/lib/systemd/user-preset/50-eruption-audio-proxy.preset
	-@rm /usr/lib/systemd/user/eruption-process-monitor.service
	-@rm /usr/lib/systemd/user-preset/50-eruption-process-monitor.preset
	-@rm /usr/lib/systemd/system/eruption-hotplug-helper.service
	-@rm /usr/lib/systemd/system-preset/50-eruption-hotplug-helper.preset
	-@rm /usr/lib/udev/rules.d/99-eruption.rules
	-@rm /usr/share/dbus-1/system.d/org.eruption.control.conf
	-@rm /usr/share/dbus-1/session.d/org.eruption.process_monitor.conf
	-@rm /usr/share/polkit-1/actions/org.eruption.policy
	-@rm /usr/share/man/man8/eruption.8
	-@rm /usr/share/man/man8/eruption-cmd.8
	-@rm /usr/share/man/man5/eruption.conf.5
	-@rm /usr/share/man/man5/process-monitor.conf.5
	-@rm /usr/share/man/man1/eruptionctl.1
	-@rm /usr/share/man/man8/eruption-hwutil.8
	-@rm /usr/share/man/man1/eruption-netfx.1
	-@rm /usr/share/man/man1/eruption-audio-proxy.1
	-@rm /usr/share/man/man1/eruption-process-monitor.1

	-@rm /usr/share/bash-completion/completions/eruption-cmd
	-@rm /usr/share/bash-completion/completions/eruption-hwutil
	-@rm /usr/share/bash-completion/completions/eruption-debug-tool
	-@rm /usr/share/bash-completion/completions/eruption-netfx
	-@rm /usr/share/bash-completion/completions/eruption-audio-proxy
	-@rm /usr/share/bash-completion/completions/eruption-process-monitor
	-@rm /usr/share/bash-completion/completions/eruptionctl
	-@rm /usr/share/fish/completions/eruption-cmd.fish
	-@rm /usr/share/fish/completions/eruption-hwutil.fish
	-@rm /usr/share/fish/completions/eruption-debug-tool.fish
	-@rm /usr/share/fish/completions/eruption-netfx.fish
	-@rm /usr/share/fish/completions/eruption-audio-proxy.fish
	-@rm /usr/share/fish/completions/eruption-process-monitor.fish
	-@rm /usr/share/fish/completions/eruptionctl.fish
	-@rm /usr/share/zsh/site-functions/_eruption-cmd
	-@rm /usr/share/zsh/site-functions/_eruption-hwutil
	-@rm /usr/share/zsh/site-functions/_eruption-debug-tool
	-@rm /usr/share/zsh/site-functions/_eruption-netfx
	-@rm /usr/share/zsh/site-functions/_eruption-audio-proxy
	-@rm /usr/share/zsh/site-functions/_eruption-process-monitor
	-@rm /usr/share/zsh/site-functions/_eruptionctl

	-@rm /usr/share/eruption/sfx/typewriter1.wav
	-@rm /usr/share/eruption/sfx/phaser1.wav
	-@rm /usr/share/eruption/sfx/phaser2.wav

	-@rm -fr /etc/eruption
	-@rm -fr /usr/share/eruption
	-@rm -fr /usr/share/eruption-gui
	-@rm -fr /var/lib/eruption

	@echo "Shutting down running Eruption daemons..."

	-@systemctl daemon-reload
	-@systemctl --global daemon-reload

	-@systemctl reload systemd-udevd

	-@systemctl --global disable --now eruption-audio-proxy.service
	-@systemctl --global disable --now eruption-process-monitor.service
	-@systemctl disable --now eruption.service

	@echo ""
	@echo "Successfully uninstalled Eruption"
	@echo ""

check:
	@cargo check

clean:
	@cargo clean

.PHONY: check clean all install uninstall build
