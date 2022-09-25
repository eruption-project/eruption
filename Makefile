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

SUDO := sudo

all: build

build:
	@cargo build $(BUILDFLAGS)

	@echo ""
	@echo "Now please run 'sudo make install' to install Eruption"
	@echo ""
	@echo "If Eruption is already running, stop it first.  Consider:"
	@echo "'make stop && sudo make install && make start'"

start:
	@echo "Notifying system daemons about Eruption..."

	-@$(SUDO) systemctl daemon-reload
	-@systemctl --user daemon-reload

	-@$(SUDO) systemctl reload systemd-udevd
	-@$(SUDO) systemctl reload dbus.service

	@echo "Starting up Eruption daemons..."

	-@systemctl --user enable --now eruption-audio-proxy.service
	-@systemctl --user enable --now eruption-process-monitor.service

	-@$(SUDO) systemctl unmask eruption.service
	-@$(SUDO) systemctl enable --now eruption.service

stop:
	@echo "Notifying system daemons about Eruption..."

	-@$(SUDO) systemctl daemon-reload
	-@systemctl --user daemon-reload

	-@$(SUDO) systemctl reload systemd-udevd
	-@$(SUDO) systemctl reload dbus.service

	@echo "Shutting down daemons..."

	-@systemctl --user disable --now eruption-audio-proxy.service
	-@systemctl --user disable --now eruption-process-monitor.service

	-@$(SUDO) systemctl mask eruption.service
	-@$(SUDO) systemctl disable --now eruption.service

install:
	# Please be sure that all Eruption daemons have been shut down completely!
	# Otherwise there will be errors during installation (file busy)

	@echo ""
	@echo "Commencing installation of Eruption..."

	@mkdir -p "/etc/eruption"
	@mkdir -p "$(TARGET_DIR)/share/doc/eruption"
	@mkdir -p $(TARGET_DIR)/share/eruption/scripts/{lib/{macros,keymaps,themes,hwdevices/{keyboards,mice}},examples}
	@mkdir -p "$(TARGET_DIR)/share/applications"
	@mkdir -p "$(TARGET_DIR)/share/icons/hicolor/64x64/apps"
	@mkdir -p "$(TARGET_DIR)/share/eruption-gui/schemas"
	@mkdir -p "/var/lib/eruption/profiles"
	@mkdir -p "$(TARGET_DIR)/lib/systemd/system"
	@mkdir -p "$(TARGET_DIR)/lib/systemd/system-preset"
	@mkdir -p "$(TARGET_DIR)/lib/systemd/user"
	@mkdir -p "$(TARGET_DIR)/lib/systemd/user-preset"
	@mkdir -p "$(TARGET_DIR)/lib/systemd/system-sleep"
	@mkdir -p "$(TARGET_DIR)/lib/udev/rules.d/"
	@mkdir -p "$(TARGET_DIR)/share/dbus-1/system.d"
	@mkdir -p "$(TARGET_DIR)/share/dbus-1/session.d"
	@mkdir -p "$(TARGET_DIR)/share/polkit-1/actions"
	@mkdir -p "$(TARGET_DIR)/share/man/man8"
	@mkdir -p "$(TARGET_DIR)/share/man/man5"
	@mkdir -p "$(TARGET_DIR)/share/man/man1"
	@mkdir -p "$(TARGET_DIR)/share/bash-completion/completions"
	@mkdir -p "$(TARGET_DIR)/share/fish/completions"
	@mkdir -p "$(TARGET_DIR)/share/zsh/site-functions"
	@mkdir -p "$(TARGET_DIR)/share/eruption/i18n"
	@mkdir -p "$(TARGET_DIR)/share/eruption/sfx"

	@cp "support/assets/eruption-gui/eruption-gui.desktop" "$(TARGET_DIR)/share/applications/"
	@cp "support/assets/eruption-gui/eruption-gui.png" "$(TARGET_DIR)/share/icons/hicolor/64x64/apps/"
	@cp "eruption-gui/schemas/gschemas.compiled" "$(TARGET_DIR)/share/eruption-gui/schemas/"
	@cp "support/systemd/eruption-suspend.sh" "$(TARGET_DIR)/lib/systemd/system-sleep/eruption"
	@cp "support/config/eruption.conf" "/etc/eruption/"
	@cp "support/config/audio-proxy.conf" "/etc/eruption/"
	@cp "support/config/process-monitor.conf" "/etc/eruption/"
	@cp "support/systemd/eruption.service" "$(TARGET_DIR)/lib/systemd/system/"
	@cp "support/systemd/eruption.preset" "$(TARGET_DIR)/lib/systemd/system-preset/50-eruption.preset"
	@cp "support/systemd/eruption-audio-proxy.service" "$(TARGET_DIR)/lib/systemd/user/"
	@cp "support/systemd/eruption-audio-proxy.preset" "$(TARGET_DIR)/lib/systemd/user-preset/50-eruption-audio-proxy.preset"
	@cp "support/systemd/eruption-process-monitor.service" "$(TARGET_DIR)/lib/systemd/user/"
	@cp "support/systemd/eruption-process-monitor.preset" "$(TARGET_DIR)/lib/systemd/user-preset/50-eruption-process-monitor.preset"
	@cp "support/systemd/eruption-hotplug-helper.service" "$(TARGET_DIR)/lib/systemd/system/"
	@cp "support/systemd/eruption-hotplug-helper.preset" "$(TARGET_DIR)/lib/systemd/system-preset/50-eruption-hotplug-helper.preset"
	@cp "support/udev/99-eruption.rules" "$(TARGET_DIR)/lib/udev/rules.d/"
	@cp "support/dbus/org.eruption.control.conf" "$(TARGET_DIR)/share/dbus-1/system.d/"
	@cp "support/dbus/org.eruption.process_monitor.conf" "$(TARGET_DIR)/share/dbus-1/session.d/"
	@cp "support/policykit/org.eruption.policy" "$(TARGET_DIR)/share/polkit-1/actions/"
	@cp "support/man/eruption.8" "$(TARGET_DIR)/share/man/man8/"
	@cp "support/man/eruption-cmd.8" "$(TARGET_DIR)/share/man/man8/"
	@cp "support/man/eruption.conf.5" "$(TARGET_DIR)/share/man/man5/"
	@cp "support/man/process-monitor.conf.5" "$(TARGET_DIR)/share/man/man5/"
	@cp "support/man/eruptionctl.1" "$(TARGET_DIR)/share/man/man1/"
	@cp "support/man/eruption-hwutil.8" "$(TARGET_DIR)/share/man/man8/"
	@cp "support/man/eruption-keymap.1" "$(TARGET_DIR)/share/man/man1/"
	@cp "support/man/eruption-netfx.1" "$(TARGET_DIR)/share/man/man1/"
	@cp "support/man/eruption-audio-proxy.1" "$(TARGET_DIR)/share/man/man1/"
	@cp "support/man/eruption-process-monitor.1" "$(TARGET_DIR)/share/man/man1/"
	@cp "support/shell/completions/en_US/eruption-cmd.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-cmd"
	@cp "support/shell/completions/en_US/eruption-hwutil.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-hwutil"
	@cp "support/shell/completions/en_US/eruption-debug-tool.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-debug-tool"
	@cp "support/shell/completions/en_US/eruption-keymap.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-keymap"
	@cp "support/shell/completions/en_US/eruption-netfx.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-netfx"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-audio-proxy"
	@cp "support/shell/completions/en_US/eruption-process-monitor.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruption-process-monitor"
	@cp "support/shell/completions/en_US/eruptionctl.bash-completion" "$(TARGET_DIR)/share/bash-completion/completions/eruptionctl"
	@cp "support/shell/completions/en_US/eruption-cmd.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-cmd.fish"
	@cp "support/shell/completions/en_US/eruption-hwutil.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-hwutil.fish"
	@cp "support/shell/completions/en_US/eruption-debug-tool.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-debug-tool.fish"
	@cp "support/shell/completions/en_US/eruption-keymap.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-keymap.fish"
	@cp "support/shell/completions/en_US/eruption-netfx.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-netfx.fish"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-audio-proxy.fish"
	@cp "support/shell/completions/en_US/eruption-process-monitor.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruption-process-monitor.fish"
	@cp "support/shell/completions/en_US/eruptionctl.fish-completion" "$(TARGET_DIR)/share/fish/completions/eruptionctl.fish"
	@cp "support/shell/completions/en_US/eruption-cmd.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-cmd"
	@cp "support/shell/completions/en_US/eruption-hwutil.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-hwutil"
	@cp "support/shell/completions/en_US/eruption-debug-tool.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-debug-tool"
	@cp "support/shell/completions/en_US/eruption-keymap.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-keymap"
	@cp "support/shell/completions/en_US/eruption-netfx.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-netfx"
	@cp "support/shell/completions/en_US/eruption-audio-proxy.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-audio-proxy"
	@cp "support/shell/completions/en_US/eruption-process-monitor.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruption-process-monitor"
	@cp "support/shell/completions/en_US/eruptionctl.zsh-completion" "$(TARGET_DIR)/share/zsh/site-functions/_eruptionctl"
	@cp "support/sfx/typewriter1.wav" "$(TARGET_DIR)/share/eruption/sfx/"
	@cp "support/sfx/phaser1.wav" "$(TARGET_DIR)/share/eruption/sfx/"
	@cp "support/sfx/phaser2.wav" "$(TARGET_DIR)/share/eruption/sfx/"

	@chmod 0755 $(TARGET_DIR)/lib/systemd/system-sleep/eruption

	@ln -fs "phaser1.wav" "$(TARGET_DIR)/share/eruption/sfx/key-down.wav"
	@ln -fs "phaser2.wav" "$(TARGET_DIR)/share/eruption/sfx/key-up.wav"

	@cp -r eruption/src/scripts/* $(TARGET_DIR)/share/eruption/scripts/
	@cp -r support/profiles/* /var/lib/eruption/profiles/

	@cp target/release/eruption $(TARGET_DIR)/bin/
	@cp target/release/eruptionctl $(TARGET_DIR)/bin/
	@cp target/release/eruption-cmd $(TARGET_DIR)/bin/
	@cp target/release/eruption-hwutil $(TARGET_DIR)/bin/
	@cp target/release/eruption-keymap $(TARGET_DIR)/bin/
	@cp target/release/eruption-netfx $(TARGET_DIR)/bin/
	@cp target/release/eruption-debug-tool $(TARGET_DIR)/bin/
	@cp target/release/eruption-hotplug-helper $(TARGET_DIR)/bin/
	@cp target/release/eruption-util $(TARGET_DIR)/bin/
	@cp target/release/eruption-gui $(TARGET_DIR)/bin/
	@cp target/release/eruption-audio-proxy $(TARGET_DIR)/bin/
	@cp target/release/eruption-process-monitor $(TARGET_DIR)/bin/

	@setcap CAP_NET_ADMIN+ep $(TARGET_DIR)/bin/eruption-process-monitor

	@echo "Successfully installed Eruption!"
	@echo ""
	@echo "Now please run 'make start' to enable Eruption"

uninstall:
	@echo "Commencing removal of Eruption..."

	-@rm $(TARGET_DIR)/bin/eruption
	-@rm $(TARGET_DIR)/bin/eruptionctl
	-@rm $(TARGET_DIR)/bin/eruption-cmd
	-@rm $(TARGET_DIR)/bin/eruption-hwutil
	-@rm $(TARGET_DIR)/bin/eruption-keymap
	-@rm $(TARGET_DIR)/bin/eruption-netfx
	-@rm $(TARGET_DIR)/bin/eruption-debug-tool
	-@rm $(TARGET_DIR)/bin/eruption-hotplug-helper
	-@rm $(TARGET_DIR)/bin/eruption-util
	-@rm $(TARGET_DIR)/bin/eruption-gui
	-@rm $(TARGET_DIR)/bin/eruption-audio-proxy
	-@rm $(TARGET_DIR)/bin/eruption-process-monitor

	-@rm $(TARGET_DIR)/share/applications/eruption-gui.desktop
	-@rm $(TARGET_DIR)/share/icons/hicolor/64x64/apps/eruption-gui.png
	-@rm $(TARGET_DIR)/share/eruption-gui/schemas/gschemas.compiled
	-@rm $(TARGET_DIR)/lib/systemd/system-sleep/eruption
	-@rm $(TARGET_DIR)/lib/systemd/system/eruption.service
	-@rm $(TARGET_DIR)/lib/systemd/system-preset/50-eruption.preset
	-@rm $(TARGET_DIR)/lib/systemd/user/eruption-audio-proxy.service
	-@rm $(TARGET_DIR)/lib/systemd/user-preset/50-eruption-audio-proxy.preset
	-@rm $(TARGET_DIR)/lib/systemd/user/eruption-process-monitor.service
	-@rm $(TARGET_DIR)/lib/systemd/user-preset/50-eruption-process-monitor.preset
	-@rm $(TARGET_DIR)/lib/systemd/system/eruption-hotplug-helper.service
	-@rm $(TARGET_DIR)/lib/systemd/system-preset/50-eruption-hotplug-helper.preset
	-@rm $(TARGET_DIR)/lib/udev/rules.d/99-eruption.rules
	-@rm $(TARGET_DIR)/share/dbus-1/system.d/org.eruption.control.conf
	-@rm $(TARGET_DIR)/share/dbus-1/session.d/org.eruption.process_monitor.conf
	-@rm $(TARGET_DIR)/share/polkit-1/actions/org.eruption.policy
	-@rm $(TARGET_DIR)/share/man/man8/eruption.8
	-@rm $(TARGET_DIR)/share/man/man8/eruption-cmd.8
	-@rm $(TARGET_DIR)/share/man/man5/eruption.conf.5
	-@rm $(TARGET_DIR)/share/man/man5/process-monitor.conf.5
	-@rm $(TARGET_DIR)/share/man/man1/eruptionctl.1
	-@rm $(TARGET_DIR)/share/man/man8/eruption-hwutil.8
	-@rm $(TARGET_DIR)/share/man/man1/eruption-netfx.1
	-@rm $(TARGET_DIR)/share/man/man1/eruption-keymap.1
	-@rm $(TARGET_DIR)/share/man/man1/eruption-audio-proxy.1
	-@rm $(TARGET_DIR)/share/man/man1/eruption-process-monitor.1

	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-cmd
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-hwutil
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-debug-tool
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-keymap
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-netfx
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-audio-proxy
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruption-process-monitor
	-@rm $(TARGET_DIR)/share/bash-completion/completions/eruptionctl
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-cmd.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-hwutil.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-debug-tool.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-keymap.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-netfx.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-audio-proxy.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruption-process-monitor.fish
	-@rm $(TARGET_DIR)/share/fish/completions/eruptionctl.fish
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-cmd
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-hwutil
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-debug-tool
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-keymap
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-netfx
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-audio-proxy
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruption-process-monitor
	-@rm $(TARGET_DIR)/share/zsh/site-functions/_eruptionctl

	-@rm $(TARGET_DIR)/share/eruption/sfx/typewriter1.wav
	-@rm $(TARGET_DIR)/share/eruption/sfx/phaser1.wav
	-@rm $(TARGET_DIR)/share/eruption/sfx/phaser2.wav

	-@rm -fr /etc/eruption
	-@rm -fr $(TARGET_DIR)/share/eruption
	-@rm -fr $(TARGET_DIR)/share/eruption-gui
	-@rm -fr /var/lib/eruption

	@echo "Successfully uninstalled Eruption!"

check:
	@cargo check

clean:
	@cargo clean

.PHONY: check clean all start stop install uninstall build
