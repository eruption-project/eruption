%global OrigName eruption
%global ShortName eruption

Name:    eruption
Version: 0.1.23
Release: 1%{?dist}
Summary: Eruption - Linux user-mode input and LED driver for keyboards, mice and other devices
URL:     https://github.com/X3n0m0rph59/eruption
License: GPLv3+

Source0: https://github.com/X3n0m0rph59/%{OrigName}/archive/releases/v0.1.23/v0.1.23.tar.gz

BuildRoot: %{_tmppath}/%{name}-build

BuildRequires: cargo
BuildRequires: systemd-devel
BuildRequires: dbus-devel
BuildRequires: hidapi-devel
BuildRequires: libevdev-devel
BuildRequires: libusbx-devel
BuildRequires: pulseaudio-libs-devel
BuildRequires: lua-devel
BuildRequires: libX11-devel
BuildRequires: libXrandr-devel
#BuildRequires: gtk3-devel
#BuildRequires: gtksourceview3-devel

Requires: systemd
Requires: dbus
Requires: hidapi
Requires: libevdev
Requires: luajit
#Requires: gtksourceview3

Recommends: lua-socket-compat

Conflicts: eruption-git
Conflicts: eruption-roccat-vulcan
Conflicts: eruption-roccat-vulcan-git

%global gittag master
%global debug_package %{nil}

%description
Linux user-mode input and LED driver for keyboards, mice and other devices

%prep
%autosetup -v -n eruption-releases-v%{version}

%build
cargo build --release --verbose

%install
%{__mkdir_p} %{buildroot}%{_mandir}/man5
%{__mkdir_p} %{buildroot}%{_mandir}/man8
%{__mkdir_p} %{buildroot}%{_mandir}/man1
%{__mkdir_p} %{buildroot}%{_sysconfdir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_sysconfdir}/dbus-1/system.d
%{__mkdir_p} %{buildroot}%{_sysconfdir}/dbus-1/session.d
%{__mkdir_p} %{buildroot}/usr/lib/udev/rules.d
%{__mkdir_p} %{buildroot}%{_datarootdir}/polkit-1/actions/
%{__mkdir_p} %{buildroot}/usr/lib/systemd/system-sleep
%{__mkdir_p} %{buildroot}%{_unitdir}
%{__mkdir_p} %{buildroot}%{_presetdir}
%{__mkdir_p} %{buildroot}%{_userunitdir}
%{__mkdir_p} %{buildroot}%{_userpresetdir}
%{__mkdir_p} %{buildroot}%{_sharedstatedir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib/macros
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib/themes
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib/hwdevices/keyboards
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib/hwdevices/mice
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/lib/hwdevices/misc
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts/examples
%{__mkdir_p} %{buildroot}%{_docdir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_datarootdir}/icons/hicolor/scalable/apps
%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/sfx
%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/i18n
%{__mkdir_p} %{buildroot}%{_datarootdir}/applications/
%{__mkdir_p} %{buildroot}%{_datarootdir}/icons/hicolor/64x64/apps/
#%{__mkdir_p} %{buildroot}%{_datarootdir}/eruption-gui/schemas
%{__mkdir_p} %{buildroot}%{_datarootdir}/bash-completion/completions/
%{__mkdir_p} %{buildroot}%{_datarootdir}/fish/completions/
%{__mkdir_p} %{buildroot}%{_datarootdir}/zsh/site-functions/

cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption.8 %{buildroot}/%{_mandir}/man8/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption-cmd.8 %{buildroot}/%{_mandir}/man8/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption-hwutil.8 %{buildroot}/%{_mandir}/man8/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption.conf.5 %{buildroot}/%{_mandir}/man5/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/process-monitor.conf.5 %{buildroot}/%{_mandir}/man5/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruptionctl.1 %{buildroot}/%{_mandir}/man1/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption-netfx.1 %{buildroot}/%{_mandir}/man1/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption-audio-proxy.1 %{buildroot}/%{_mandir}/man1/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/man/eruption-process-monitor.1 %{buildroot}/%{_mandir}/man1/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-debug-tool.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-debug-tool
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-cmd.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-cmd
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-hwutil.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-hwutil
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-netfx.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-netfx
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-audio-proxy.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-audio-proxy
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-process-monitor.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruption-process-monitor
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruptionctl.bash-completion %{buildroot}/%{_datarootdir}/bash-completion/completions/eruptionctl
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-debug-tool.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-debug-tool.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-cmd.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-cmd.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-hwutil.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-hwutil.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-netfx.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-netfx.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-audio-proxy.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-audio-proxy.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-process-monitor.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruption-process-monitor.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruptionctl.fish-completion %{buildroot}/%{_datarootdir}/fish/completions/eruptionctl.fish
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-debug-tool.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-debug-tool
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-cmd.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-cmd
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-hwutil.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-hwutil
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-netfx.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-netfx
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-audio-proxy.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-audio-proxy
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruption-process-monitor.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruption-process-monitor
cp -a %{_builddir}/%{name}-releases-v%{version}/support/shell/completions/en_US/eruptionctl.zsh-completion %{buildroot}/%{_datarootdir}/zsh/site-functions/_eruptionctl
cp -a %{_builddir}/%{name}-releases-v%{version}/support/config/eruption.conf %{buildroot}/%{_sysconfdir}/%{ShortName}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/config/audio-proxy.conf %{buildroot}/%{_sysconfdir}/%{ShortName}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/config/process-monitor.conf %{buildroot}/%{_sysconfdir}/%{ShortName}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/dbus/org.eruption.control.conf %{buildroot}%{_sysconfdir}/dbus-1/system.d/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/dbus/org.eruption.process_monitor.conf %{buildroot}%{_sysconfdir}/dbus-1/session.d/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/udev/99-eruption.rules %{buildroot}/usr/lib/udev/rules.d/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/policykit/org.eruption.policy %{buildroot}%{_datarootdir}/polkit-1/actions/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption.preset %{buildroot}/%{_presetdir}/50-eruption.preset
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption.service %{buildroot}/%{_unitdir}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-audio-proxy.preset %{buildroot}/%{_userpresetdir}/50-eruption-audio-proxy.preset
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-audio-proxy.service %{buildroot}/%{_userunitdir}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-process-monitor.preset %{buildroot}/%{_userpresetdir}/50-eruption-process-monitor.preset
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-process-monitor.service %{buildroot}/%{_userunitdir}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-hotplug-helper.preset %{buildroot}/%{_presetdir}/50-eruption-hotplug-helper.preset
cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-hotplug-helper.service %{buildroot}/%{_unitdir}/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/animal-blobby.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/animal-blobby-swirl.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/animal-breathing-1.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/animal-breathing-2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/animal-breathing-3.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/audio-visualization-1.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/audio-visualization-2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/audio-visualization-3.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/audio-visualization-4.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/audio-visualization-5.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/domain-coloring-1.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/domain-coloring-2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/domain-coloring-3.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/default.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/checkerboard.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/fx1.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/fx2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/fireplace.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/fireworks.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/flight-perlin.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/gaming.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/gradient-noise.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/heartbeat-sysmon.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/heatmap.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/heatmap-errors.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/lava-lamp.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/lava-lamp-pastel.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/matrix.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/netfx.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/batique.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/batique-mouse.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/blackout.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/blue-fx-swirl-perlin.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/profile1.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/profile2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/profile3.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/profile4.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/psychedelic.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/twinkle.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/rainbow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/preset-red-yellow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/preset-blue-red.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/rainbow-wave.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/red-fx.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/red-wave.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/ripple-rainbow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/snake.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/solid-wave.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/solid.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/starcraft2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/spectrum-analyzer.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/spectrum-analyzer-swirl.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/vu-meter.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-blue-red.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-rainbow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-red-yellow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-dim.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-blue-red-dim.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-rainbow-dim.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-perlin-red-yellow-dim.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-simplex-rainbow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-turbulence.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/swirl-voronoi.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/profiles/turbulence.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-releases-v%{version}/support/sfx/typewriter1.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/typewriter1.wav
cp -a %{_builddir}/%{name}-releases-v%{version}/support/sfx/phaser1.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/phaser1.wav
cp -a %{_builddir}/%{name}-releases-v%{version}/support/sfx/phaser2.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/phaser2.wav
ln -s phaser1.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/key-down.wav
ln -s phaser2.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/key-up.wav

cp -ra %{_builddir}/%{name}-releases-v%{version}/eruption/src/scripts %{buildroot}%{_datarootdir}/%{ShortName}/

cp -a %{_builddir}/%{name}-releases-v%{version}/support/systemd/eruption-suspend.sh %{buildroot}/usr/lib/systemd/system-sleep/eruption

#cp -a %{_builddir}/%{name}-releases-v%{version}/support/assets/eruption-gui/eruption-gui.desktop %{buildroot}/usr/share/applications/eruption-gui.desktop
#cp -a %{_builddir}/%{name}-releases-v%{version}/support/assets/eruption-gui/eruption-gui.png %{buildroot}/usr/share/icons/hicolor/64x64/apps/eruption-gui.png
#cp -a %{_builddir}/%{name}-releases-v%{version}/eruption-gui/schemas/gschemas.compiled %{buildroot}/usr/share/eruption-gui/schemas/

install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption %{buildroot}%{_bindir}/eruption
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruptionctl %{buildroot}%{_bindir}/eruptionctl
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-cmd %{buildroot}%{_bindir}/eruption-cmd
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-hwutil %{buildroot}%{_bindir}/eruption-hwutil
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-netfx %{buildroot}%{_bindir}/eruption-netfx
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-util %{buildroot}%{_bindir}/eruption-util
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-debug-tool %{buildroot}%{_bindir}/eruption-debug-tool
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-hotplug-helper %{buildroot}%{_bindir}/eruption-hotplug-helper
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-audio-proxy %{buildroot}%{_bindir}/eruption-audio-proxy
install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-process-monitor %{buildroot}%{_bindir}/eruption-process-monitor
#install -Dp -m 0755 %{_builddir}/%{name}-releases-v%{version}/target/release/eruption-gui %{buildroot}%{_bindir}/eruption-gui

%post
%systemd_post eruption.service
%systemd_user_post eruption-audio-proxy.service
%systemd_user_post eruption-process-monitor.service

%preun
%systemd_preun eruption.service
%systemd_user_preun eruption-audio-proxy.service
%systemd_user_preun eruption-process-monitor.service

%postun
%systemd_postun_with_restart eruption.service
%systemd_user_postun_with_restart eruption-audio-proxy.service
%systemd_user_postun_with_restart eruption-process-monitor.service

%files
%doc %{_mandir}/man5/eruption.conf.5.gz
%doc %{_mandir}/man8/eruption.8.gz
%doc %{_mandir}/man8/eruption-cmd.8.gz
%doc %{_mandir}/man8/eruption-hwutil.8.gz
%doc %{_mandir}/man1/eruptionctl.1.gz
%doc %{_mandir}/man1/eruption-netfx.1.gz
%doc %{_mandir}/man1/eruption-audio-proxy.1.gz
%doc %{_mandir}/man1/eruption-process-monitor.1.gz
%doc %{_mandir}/man5/process-monitor.conf.5.gz
%dir %{_datarootdir}/icons/hicolor/scalable/apps/
%config(noreplace) %{_sysconfdir}/%{ShortName}/%{ShortName}.conf
%config(noreplace) %{_sysconfdir}/%{ShortName}/audio-proxy.conf
%config(noreplace) %{_sysconfdir}/%{ShortName}/process-monitor.conf
%{_sysconfdir}/dbus-1/system.d/org.eruption.control.conf
%{_sysconfdir}/dbus-1/session.d/org.eruption.process_monitor.conf
%{_datarootdir}/polkit-1/actions/org.eruption.policy
/usr/lib/udev/rules.d/99-eruption.rules
/usr/lib/systemd/system-sleep/eruption
%{_bindir}/eruption
%{_bindir}/eruptionctl
%{_bindir}/eruption-cmd
%{_bindir}/eruption-hwutil
%{_bindir}/eruption-netfx
%{_bindir}/eruption-util
%{_bindir}/eruption-debug-tool
%{_bindir}/eruption-hotplug-helper
%{_bindir}/eruption-audio-proxy
%caps(cap_net_admin=ep) %{_bindir}/eruption-process-monitor
%{_unitdir}/eruption.service
%{_presetdir}/50-eruption.preset
%{_userunitdir}/eruption-audio-proxy.service
%{_userpresetdir}/50-eruption-audio-proxy.preset
%{_userunitdir}/eruption-process-monitor.service
%{_userpresetdir}/50-eruption-process-monitor.preset
%{_unitdir}/eruption-hotplug-helper.service
%{_presetdir}/50-eruption-hotplug-helper.preset
#%{_bindir}/eruption-gui
#%{_datarootdir}/applications/eruption-gui.desktop
#%{_datarootdir}/icons/hicolor/64x64/apps/eruption-gui.png
#%{_datarootdir}/eruption-gui/schemas/gschemas.compiled
%{_datarootdir}/bash-completion/completions/eruption-debug-tool
%{_datarootdir}/bash-completion/completions/eruption-cmd
%{_datarootdir}/bash-completion/completions/eruption-hwutil
%{_datarootdir}/bash-completion/completions/eruption-netfx
%{_datarootdir}/bash-completion/completions/eruption-audio-proxy
%{_datarootdir}/bash-completion/completions/eruption-process-monitor
%{_datarootdir}/bash-completion/completions/eruptionctl
%{_datarootdir}/fish/completions/eruption-debug-tool.fish
%{_datarootdir}/fish/completions/eruption-cmd.fish
%{_datarootdir}/fish/completions/eruption-hwutil.fish
%{_datarootdir}/fish/completions/eruption-netfx.fish
%{_datarootdir}/fish/completions/eruption-audio-proxy.fish
%{_datarootdir}/fish/completions/eruption-process-monitor.fish
%{_datarootdir}/fish/completions/eruptionctl.fish
%{_datarootdir}/zsh/site-functions/_eruption-debug-tool
%{_datarootdir}/zsh/site-functions/_eruption-cmd
%{_datarootdir}/zsh/site-functions/_eruption-hwutil
%{_datarootdir}/zsh/site-functions/_eruption-netfx
%{_datarootdir}/zsh/site-functions/_eruption-audio-proxy
%{_datarootdir}/zsh/site-functions/_eruption-process-monitor
%{_datarootdir}/zsh/site-functions/_eruptionctl
%{_sharedstatedir}/%{ShortName}/profiles/animal-blobby.profile
%{_sharedstatedir}/%{ShortName}/profiles/animal-blobby-swirl.profile
%{_sharedstatedir}/%{ShortName}/profiles/animal-breathing-1.profile
%{_sharedstatedir}/%{ShortName}/profiles/animal-breathing-2.profile
%{_sharedstatedir}/%{ShortName}/profiles/animal-breathing-3.profile
%{_sharedstatedir}/%{ShortName}/profiles/audio-visualization-1.profile
%{_sharedstatedir}/%{ShortName}/profiles/audio-visualization-2.profile
%{_sharedstatedir}/%{ShortName}/profiles/audio-visualization-3.profile
%{_sharedstatedir}/%{ShortName}/profiles/audio-visualization-4.profile
%{_sharedstatedir}/%{ShortName}/profiles/audio-visualization-5.profile
%{_sharedstatedir}/%{ShortName}/profiles/domain-coloring-1.profile
%{_sharedstatedir}/%{ShortName}/profiles/domain-coloring-2.profile
%{_sharedstatedir}/%{ShortName}/profiles/domain-coloring-3.profile
%{_sharedstatedir}/%{ShortName}/profiles/default.profile
%{_sharedstatedir}/%{ShortName}/profiles/checkerboard.profile
%{_sharedstatedir}/%{ShortName}/profiles/fx1.profile
%{_sharedstatedir}/%{ShortName}/profiles/fx2.profile
%{_sharedstatedir}/%{ShortName}/profiles/fireplace.profile
%{_sharedstatedir}/%{ShortName}/profiles/fireworks.profile
%{_sharedstatedir}/%{ShortName}/profiles/flight-perlin.profile
%{_sharedstatedir}/%{ShortName}/profiles/gaming.profile
%{_sharedstatedir}/%{ShortName}/profiles/gradient-noise.profile
%{_sharedstatedir}/%{ShortName}/profiles/heartbeat-sysmon.profile
%{_sharedstatedir}/%{ShortName}/profiles/heatmap.profile
%{_sharedstatedir}/%{ShortName}/profiles/heatmap-errors.profile
%{_sharedstatedir}/%{ShortName}/profiles/lava-lamp.profile
%{_sharedstatedir}/%{ShortName}/profiles/lava-lamp-pastel.profile
%{_sharedstatedir}/%{ShortName}/profiles/matrix.profile
%{_sharedstatedir}/%{ShortName}/profiles/netfx.profile
%{_sharedstatedir}/%{ShortName}/profiles/batique.profile
%{_sharedstatedir}/%{ShortName}/profiles/batique-mouse.profile
%{_sharedstatedir}/%{ShortName}/profiles/blackout.profile
%{_sharedstatedir}/%{ShortName}/profiles/blue-fx-swirl-perlin.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile1.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile2.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile3.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile4.profile
%{_sharedstatedir}/%{ShortName}/profiles/psychedelic.profile
%{_sharedstatedir}/%{ShortName}/profiles/twinkle.profile
%{_sharedstatedir}/%{ShortName}/profiles/rainbow.profile
%{_sharedstatedir}/%{ShortName}/profiles/preset-red-yellow.profile
%{_sharedstatedir}/%{ShortName}/profiles/preset-blue-red.profile
%{_sharedstatedir}/%{ShortName}/profiles/rainbow-wave.profile
%{_sharedstatedir}/%{ShortName}/profiles/red-fx.profile
%{_sharedstatedir}/%{ShortName}/profiles/red-wave.profile
%{_sharedstatedir}/%{ShortName}/profiles/ripple-rainbow.profile
%{_sharedstatedir}/%{ShortName}/profiles/snake.profile
%{_sharedstatedir}/%{ShortName}/profiles/solid-wave.profile
%{_sharedstatedir}/%{ShortName}/profiles/solid.profile
%{_sharedstatedir}/%{ShortName}/profiles/starcraft2.profile
%{_sharedstatedir}/%{ShortName}/profiles/spectrum-analyzer.profile
%{_sharedstatedir}/%{ShortName}/profiles/spectrum-analyzer-swirl.profile
%{_sharedstatedir}/%{ShortName}/profiles/vu-meter.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-blue-red.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-rainbow.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-red-yellow.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-dim.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-blue-red-dim.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-rainbow-dim.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-perlin-red-yellow-dim.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-simplex-rainbow.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-turbulence.profile
%{_sharedstatedir}/%{ShortName}/profiles/swirl-voronoi.profile
%{_sharedstatedir}/%{ShortName}/profiles/turbulence.profile
%{_datarootdir}/%{ShortName}/scripts/examples/simple.lua
%{_datarootdir}/%{ShortName}/scripts/lib/debug.lua
%{_datarootdir}/%{ShortName}/scripts/lib/easing.lua
%{_datarootdir}/%{ShortName}/scripts/lib/queue.lua
%{_datarootdir}/%{ShortName}/scripts/lib/utilities.lua
%{_datarootdir}/%{ShortName}/scripts/lib/declarations.lua
%{_datarootdir}/%{ShortName}/scripts/lib/failsafe.lua
%{_datarootdir}/%{ShortName}/scripts/lib/failsafe.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/generic_keyboard.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/roccat_vulcan_1xx.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/roccat_vulcan_tkl.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/roccat_vulcan_pro_tkl.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/roccat_vulcan_pro.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/roccat_magma.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/keyboards/corsair_strafe.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/generic_mouse.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kone_aimo.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kone_xtd.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kain_100.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kain_2xx.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_burst_pro.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kone_aimo_remastered.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kone_pure_ultra.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kone_pro_air.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kova_aimo.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_kova_2016.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/mice/roccat_nyth.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/misc/roccat_elo_71_air.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/misc/roccat_aimo_pad.lua
%{_datarootdir}/%{ShortName}/scripts/lib/hwdevices/misc/custom_serial_leds.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/themes/default.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/themes/gaming.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/macros/modifiers.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/macros/user-macros.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/macros/failsafe-macros.lua
%config %{_datarootdir}/%{ShortName}/scripts/lib/macros/starcraft2.lua
%{_datarootdir}/%{ShortName}/scripts/lib/macros/examples.lua
%{_datarootdir}/%{ShortName}/scripts/macros.lua
%{_datarootdir}/%{ShortName}/scripts/macros.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/stats.lua
%{_datarootdir}/%{ShortName}/scripts/stats.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/afterglow.lua
%{_datarootdir}/%{ShortName}/scripts/afterglow.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/afterhue.lua
%{_datarootdir}/%{ShortName}/scripts/afterhue.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/animal.lua
%{_datarootdir}/%{ShortName}/scripts/animal.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/audioviz1.lua
%{_datarootdir}/%{ShortName}/scripts/audioviz1.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/audioviz2.lua
%{_datarootdir}/%{ShortName}/scripts/audioviz2.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/audioviz3.lua
%{_datarootdir}/%{ShortName}/scripts/audioviz3.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/audioviz4.lua
%{_datarootdir}/%{ShortName}/scripts/audioviz4.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/audioviz5.lua
%{_datarootdir}/%{ShortName}/scripts/audioviz5.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/billow.lua
%{_datarootdir}/%{ShortName}/scripts/billow.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/checkerboard.lua
%{_datarootdir}/%{ShortName}/scripts/checkerboard.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/organic.lua
%{_datarootdir}/%{ShortName}/scripts/organic.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/batique.lua
%{_datarootdir}/%{ShortName}/scripts/batique.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/fbm.lua
%{_datarootdir}/%{ShortName}/scripts/fbm.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/perlin.lua
%{_datarootdir}/%{ShortName}/scripts/perlin.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/phonon.lua
%{_datarootdir}/%{ShortName}/scripts/phonon.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/psychedelic.lua
%{_datarootdir}/%{ShortName}/scripts/psychedelic.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/pulse.lua
%{_datarootdir}/%{ShortName}/scripts/pulse.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/rmf.lua
%{_datarootdir}/%{ShortName}/scripts/rmf.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/voronoi.lua
%{_datarootdir}/%{ShortName}/scripts/voronoi.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/fire.lua
%{_datarootdir}/%{ShortName}/scripts/fire.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/fireworks.lua
%{_datarootdir}/%{ShortName}/scripts/fireworks.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/gaming.lua
%{_datarootdir}/%{ShortName}/scripts/gaming.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/ghost.lua
%{_datarootdir}/%{ShortName}/scripts/ghost.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/gradient.lua
%{_datarootdir}/%{ShortName}/scripts/gradient.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/linear-gradient.lua
%{_datarootdir}/%{ShortName}/scripts/linear-gradient.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/halo.lua
%{_datarootdir}/%{ShortName}/scripts/halo.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/heartbeat.lua
%{_datarootdir}/%{ShortName}/scripts/heartbeat.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/heatmap.lua
%{_datarootdir}/%{ShortName}/scripts/heatmap.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/impact.lua
%{_datarootdir}/%{ShortName}/scripts/impact.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/lava-lamp.lua
%{_datarootdir}/%{ShortName}/scripts/lava-lamp.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/multigradient.lua
%{_datarootdir}/%{ShortName}/scripts/multigradient.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/netfx.lua
%{_datarootdir}/%{ShortName}/scripts/netfx.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/osn.lua
%{_datarootdir}/%{ShortName}/scripts/osn.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/rainbow.lua
%{_datarootdir}/%{ShortName}/scripts/rainbow.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/raindrops.lua
%{_datarootdir}/%{ShortName}/scripts/raindrops.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/shockwave.lua
%{_datarootdir}/%{ShortName}/scripts/shockwave.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/ripple.lua
%{_datarootdir}/%{ShortName}/scripts/ripple.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/solid.lua
%{_datarootdir}/%{ShortName}/scripts/solid.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/stripes.lua
%{_datarootdir}/%{ShortName}/scripts/stripes.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/swirl-perlin.lua
%{_datarootdir}/%{ShortName}/scripts/swirl-perlin.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/swirl-simplex.lua
%{_datarootdir}/%{ShortName}/scripts/swirl-simplex.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/swirl-turbulence.lua
%{_datarootdir}/%{ShortName}/scripts/swirl-turbulence.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/swirl-voronoi.lua
%{_datarootdir}/%{ShortName}/scripts/swirl-voronoi.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/flight-perlin.lua
%{_datarootdir}/%{ShortName}/scripts/flight-perlin.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/sysmon.lua
%{_datarootdir}/%{ShortName}/scripts/sysmon.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/temperature.lua
%{_datarootdir}/%{ShortName}/scripts/temperature.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/turbulence.lua
%{_datarootdir}/%{ShortName}/scripts/turbulence.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/water.lua
%{_datarootdir}/%{ShortName}/scripts/water.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/wave.lua
%{_datarootdir}/%{ShortName}/scripts/wave.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/snake.lua
%{_datarootdir}/%{ShortName}/scripts/snake.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/domain-coloring.lua
%{_datarootdir}/%{ShortName}/scripts/domain-coloring.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/dim-zone.lua
%{_datarootdir}/%{ShortName}/scripts/dim-zone.lua.manifest
%{_datarootdir}/%{ShortName}/sfx/typewriter1.wav
%{_datarootdir}/%{ShortName}/sfx/phaser1.wav
%{_datarootdir}/%{ShortName}/sfx/phaser2.wav
%{_datarootdir}/%{ShortName}/sfx/key-down.wav
%{_datarootdir}/%{ShortName}/sfx/key-up.wav

%changelog
