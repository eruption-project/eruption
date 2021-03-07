# Table of Contents

- [Table of Contents](#table-of-contents)
- [How to build and install Eruption from source](#how-to-build-and-install-eruption-from-source)
    - [Install build dependencies](#install-build-dependencies)
      - [On Arch-based distros](#on-arch-based-distros)
      - [On Fedora-based distros](#on-fedora-based-distros)
      - [On Debian-based distros](#on-debian-based-distros)
    - [Clone the project and build the release binaries](#clone-the-project-and-build-the-release-binaries)
    - [Create the target directories and copy over all the required files](#create-the-target-directories-and-copy-over-all-the-required-files)
      - [1. Create the target directories](#1-create-the-target-directories)
      - [2. Copy over the base files](#2-copy-over-the-base-files)
      - [3. Copy over the binaries](#3-copy-over-the-binaries)
      - [4. Copy over scripts and profiles](#4-copy-over-scripts-and-profiles)
    - [Run Eruption](#run-eruption)

# How to build and install Eruption from source

To build Eruption from source you need to have `git` and `rust` installed, and you need to install the build dependencies of Eruption as well. You need at least the current `stable` release of `rust` (version `1.50.0`). You probably may want to use [https://rustup.rs/](https://rustup.rs/).

The list of files and directories were taken from `support/pkg/arch/PKGBUILD`, but they should be applicable to most Linux based systems.

### Install build dependencies

#### On Arch-based distros

```sh
 $ sudo pacman -Sy libevdev hidapi systemd-libs dbus libpulse luajit lua51-socket gtksourceview3
 $ sudo pacman -Sy xorg-server-devel libxrandr gtk3
```

#### On Fedora-based distros

```sh
$ sudo dnf install systemd dbus hidapi libevdev luajit gtksourceview3 lua-socket-compat
$ sudo dnf install systemd-devel dbus-devel hidapi-devel libevdev-devel libusbx-devel \
 pulseaudio-libs-devel luajit-devel libX11-devel libXrandr-devel gtk3-devel gtksourceview3-devel
```

#### On Debian-based distros

```sh
 $ sudo apt install libusb-1.0-0-dev libhidapi-dev libevdev-dev libudev-dev libdbus-1-dev \
 libpulse-dev luajit libluajit-5.1-dev libx11-dev libxrandr-dev libgtk-3-dev libgdk-pixbuf2.0-dev \
 libatk1.0-dev libpango1.0-dev libcairo2-dev libgtksourceview-3.0-dev
```

### Clone the project and build the release binaries

```sh
 $ git clone https://github.com/X3n0m0rph59/eruption.git

 $ cd eruption
 $ cargo build --all --release
```

### Create the target directories and copy over all the required files

#### 1. Create the target directories

```sh
sudo mkdir -p "/etc/eruption"
sudo mkdir -p "/usr/share/doc/eruption"
sudo mkdir -p /usr/share/eruption/scripts/{lib/{macros,themes,hwdevices/{keyboards,mice}},examples}

sudo mkdir -p "/usr/share/applications"
sudo mkdir -p "/usr/share/icons/hicolor/64x64/apps"
sudo mkdir -p "/usr/share/eruption-gui/schemas"
sudo mkdir -p "/var/lib/eruption/profiles"
sudo mkdir -p "/usr/lib/systemd/system"
sudo mkdir -p "/usr/lib/systemd/system-preset"
sudo mkdir -p "/usr/lib/systemd/user"
sudo mkdir -p "/usr/lib/systemd/user-preset"
sudo mkdir -p "/usr/lib/systemd/system-sleep"
sudo mkdir -p "/usr/lib/udev/rules.d/"
sudo mkdir -p "/usr/share/dbus-1/system.d"
sudo mkdir -p "/usr/share/dbus-1/session.d"
sudo mkdir -p "/usr/share/polkit-1/actions"
sudo mkdir -p "/usr/share/man/man8"
sudo mkdir -p "/usr/share/man/man5"
sudo mkdir -p "/usr/share/man/man1"
sudo mkdir -p "/usr/share/bash-completion/completions"
sudo mkdir -p "/usr/share/fish/completions"
sudo mkdir -p "/usr/share/zsh/site-functions"
sudo mkdir -p "/usr/share/eruption/i18n"
sudo mkdir -p "/usr/share/eruption/sfx"
```

#### 2. Copy over the base files

```sh
 sudo cp "support/assets/eruption-gui/eruption-gui.desktop" "/usr/share/applications/"
 sudo cp "support/assets/eruption-gui/eruption-gui.png" "/usr/share/icons/hicolor/64x64/apps/"
 sudo cp "eruption-gui/schemas/gschemas.compiled" "/usr/share/eruption-gui/schemas/"
 sudo cp "support/systemd/eruption-suspend.sh" "/usr/lib/systemd/system-sleep/eruption"
 sudo cp "support/config/eruption.conf" "/etc/eruption/"
 sudo cp "support/config/process-monitor.conf" "/etc/eruption/"
 sudo cp "support/systemd/eruption.service" "/usr/lib/systemd/system/"
 sudo cp "support/systemd/eruption.preset" "/usr/lib/systemd/system-preset/50-eruption.preset"
 sudo cp "support/systemd/eruption-process-monitor.service" "/usr/lib/systemd/user/"
 sudo cp "support/systemd/eruption-process-monitor.preset" "/usr/lib/systemd/user-preset/50-eruption-process-monitor.preset"
 sudo cp "support/udev/99-eruption.rules" "/usr/lib/udev/rules.d/"
 sudo cp "support/dbus/org.eruption.control.conf" "/usr/share/dbus-1/system.d/"
 sudo cp "support/dbus/org.eruption.process_monitor.conf" "/usr/share/dbus-1/session.d/"
 sudo cp "support/policykit/org.eruption.policy" "/usr/share/polkit-1/actions/"
 sudo cp "support/man/eruption.8" "/usr/share/man/man8/"
 sudo cp "support/man/eruption.conf.5" "/usr/share/man/man5/"
 sudo cp "support/man/process-monitor.conf.5" "/usr/share/man/man5/"
 sudo cp "support/man/eruptionctl.1" "/usr/share/man/man1/"
 sudo cp "support/man/eruption-netfx.1" "/usr/share/man/man1/"
 sudo cp "support/man/eruption-process-monitor.1" "/usr/share/man/man1/"

 sudo cp "support/shell/completions/en_US/eruption-debug-tool.bash-completion" "/usr/share/bash-completion/completions/eruption-debug-tool"
 sudo cp "support/shell/completions/en_US/eruption-netfx.bash-completion" "/usr/share/bash-completion/completions/eruption-netfx"
 sudo cp "support/shell/completions/en_US/eruption-process-monitor.bash-completion" "/usr/share/bash-completion/completions/eruption-process-monitor"
 sudo cp "support/shell/completions/en_US/eruptionctl.bash-completion" "/usr/share/bash-completion/completions/eruptionctl"

 sudo cp "support/shell/completions/en_US/eruption-debug-tool.fish-completion" "/usr/share/fish/completions/eruption-debug-tool.fish"
 sudo cp "support/shell/completions/en_US/eruption-netfx.fish-completion" "/usr/share/fish/completions/eruption-netfx.fish"
 sudo cp "support/shell/completions/en_US/eruption-process-monitor.fish-completion" "/usr/share/fish/completions/eruption-process-monitor.fish"
 sudo cp "support/shell/completions/en_US/eruptionctl.fish-completion" "/usr/share/fish/completions/eruptionctl.fish"

 sudo cp "support/shell/completions/en_US/eruption-debug-tool.zsh-completion" "/usr/share/zsh/site-functions/_eruption-debug-tool"
 sudo cp "support/shell/completions/en_US/eruption-netfx.zsh-completion" "/usr/share/zsh/site-functions/_eruption-netfx"
 sudo cp "support/shell/completions/en_US/eruption-process-monitor.zsh-completion" "/usr/share/zsh/site-functions/_eruption-process-monitor"
 sudo cp "support/shell/completions/en_US/eruptionctl.zsh-completion" "/usr/share/zsh/site-functions/_eruptionctl"

 sudo cp "support/sfx/typewriter1.wav" "/usr/share/eruption/sfx/"
 sudo cp "support/sfx/phaser1.wav" "/usr/share/eruption/sfx/"
 sudo cp "support/sfx/phaser2.wav" "/usr/share/eruption/sfx/"

 # Set file modes
 sudo chmod 0755 /usr/lib/systemd/system-sleep/eruption

 # Create required symlinks
 sudo ln -s "phaser1.wav" "/usr/share/eruption/sfx/key-down.wav"
 sudo ln -s "phaser2.wav" "/usr/share/eruption/sfx/key-up.wav"
```

#### 3. Copy over the binaries

```sh
 sudo cp target/release/eruption{,ctl,-netfx,-debug-tool,-gui,-process-monitor} /usr/bin/ && sudo setcap CAP_NET_ADMIN+ep /usr/bin/eruption-process-monitor
```

#### 4. Copy over scripts and profiles

```sh
 sudo cp -r eruption/src/scripts/* /usr/share/eruption/scripts/
 sudo cp -r support/profiles/* /var/lib/eruption/profiles/
```

### Run Eruption

```sh
 sudo systemctl daemon-reload
 sudo systemctl start eruption.service
```

You do not need to `enable` the systemd service, since Eruption ist started by an `udev` rule.