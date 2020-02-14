%global OrigName eruption-roccat-vulcan
%global ShortName eruption

Name:    eruption-roccat-vulcan-git
Version: 0.0.12
Release: 3%{?dist}
Summary: eruption-roccat-vulcan - Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards
URL:     https://x3n0m0rph59.gitlab.io/eruption-roccat-vulcan/
License: GPLv3+

# Source0: https://gitlab.com/X3n0m0rph59/eruption-roccat-vulcan.git
Source0: https://gitlab.com/X3n0m0rph59/%{OrigName}/-/archive/master/%{OrigName}-master.tar.gz

BuildRoot: %{_tmppath}/%{name}-build

BuildRequires: cargo
BuildRequires: systemd-devel
BuildRequires: dbus-devel
BuildRequires: hidapi-devel
BuildRequires: libevdev-devel
BuildRequires: libusbx-devel
BuildRequires: pulseaudio-libs-devel
BuildRequires: alsa-lib-devel

Requires: systemd
Requires: dbus
Requires: hidapi
Requires: libevdev

Conflicts: eruption-roccat-vulcan

%global gittag master
%global debug_package %{nil}

%description
Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards

%prep
# %autosetup -n %{name}-%{version}
%autosetup %{OrigName}-master

%build
cargo build --all --release --verbose

%install
%{__mkdir_p} %{buildroot}%{_mandir}/man5
%{__mkdir_p} %{buildroot}%{_mandir}/man8
%{__mkdir_p} %{buildroot}%{_sysconfdir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_sysconfdir}/dbus-1/system.d
%{__mkdir_p} %{buildroot}/usr/lib/udev/rules.d
%{__mkdir_p} %{buildroot}%{_unitdir}
%{__mkdir_p} %{buildroot}%{_presetdir}
%{__mkdir_p} %{buildroot}%{_sharedstatedir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles
%{__mkdir_p} %{buildroot}%{_libdir}/%{ShortName}/scripts
%{__mkdir_p} %{buildroot}%{_docdir}/%{ShortName}
%{__mkdir_p} %{buildroot}%{_datarootdir}/icons/hicolor/scalable/apps
%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/sfx
%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/i18n
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/templates
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/css
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/css/styles
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/css/themes/eruption
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/css/themes/metal
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/js
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/font
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/img
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/img/bg
#%{__mkdir_p} %{buildroot}%{_datarootdir}/%{ShortName}/static/img/icons

cp -a %{_builddir}/%{name}-%{version}/support/man/eruption.8 %{buildroot}/%{_mandir}/man8/
cp -a %{_builddir}/%{name}-%{version}/support/man/eruption.conf.5 %{buildroot}/%{_mandir}/man5/
cp -a %{_builddir}/%{name}-%{version}/support/config/eruption.conf %{buildroot}/%{_sysconfdir}/%{ShortName}/
cp -a %{_builddir}/%{name}-%{version}/support/dbus/org.eruption.control.conf %{buildroot}%{_sysconfdir}/dbus-1/system.d/
cp -a %{_builddir}/%{name}-%{version}/support/udev/99-eruption-roccat-vulcan.rules %{buildroot}/usr/lib/udev/rules.d/
cp -a %{_builddir}/%{name}-%{version}/support/systemd/eruption.preset %{buildroot}/%{_presetdir}/50-eruption.preset
cp -a %{_builddir}/%{name}-%{version}/support/systemd/eruption.service %{buildroot}/%{_unitdir}/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/default.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/gaming.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/profile2.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/profile3.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/preset-red-yellow.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
cp -a %{_builddir}/%{name}-%{version}/support/profiles/preset-blue-red.profile %{buildroot}%{_sharedstatedir}/%{ShortName}/profiles/
#cp -a %{_builddir}/%{name}-%{version}/support/sfx/typewriter1.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/typewriter1.wav
#cp -a %{_builddir}/%{name}-%{version}/support/sfx/phaser1.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/phaser1.wav
#cp -a %{_builddir}/%{name}-%{version}/support/sfx/phaser2.wav %{buildroot}%{_datarootdir}/%{ShortName}/sfx/phaser2.wav
cp -ra %{_builddir}/%{name}-%{version}/src/scripts %{buildroot}%{_datarootdir}/%{ShortName}/
#cp -ra %{_builddir}/%{name}-%{version}/templates %{buildroot}%{_datarootdir}/%{ShortName}/
#cp -ra %{_builddir}/%{name}-%{version}/static %{buildroot}%{_datarootdir}/%{ShortName}/

install -Dp -m 0755 %{_builddir}/%{name}-%{version}/target/release/eruption %{buildroot}%{_bindir}/eruption

%post
%systemd_post %{ShortName}.service

%preun
%systemd_preun %{ShortName}.service

%postun
%systemd_postun_with_restart %{ShortName}.service

%files
%doc %{_mandir}/man5/eruption.conf.5.gz
%doc %{_mandir}/man8/eruption.8.gz
%dir %{_datarootdir}/icons/hicolor/scalable/apps/
%config(noreplace) %{_sysconfdir}/%{ShortName}/%{ShortName}.conf
%{_sysconfdir}/dbus-1/system.d/org.eruption.control.conf
/usr/lib/udev/rules.d/99-eruption-roccat-vulcan.rules
%{_bindir}/eruption
%{_unitdir}/eruption.service
%{_presetdir}/50-eruption.preset
%{_sharedstatedir}/%{ShortName}/profiles/default.profile
%{_sharedstatedir}/%{ShortName}/profiles/gaming.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile2.profile
%{_sharedstatedir}/%{ShortName}/profiles/profile3.profile
%{_sharedstatedir}/%{ShortName}/profiles/preset-red-yellow.profile
%{_sharedstatedir}/%{ShortName}/profiles/preset-blue-red.profile
%{_datarootdir}/%{ShortName}/scripts/examples/simple.lua
%{_datarootdir}/%{ShortName}/scripts/lib/debug.lua
%{_datarootdir}/%{ShortName}/scripts/afterglow.lua
%{_datarootdir}/%{ShortName}/scripts/afterglow.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/afterhue.lua
%{_datarootdir}/%{ShortName}/scripts/afterhue.lua.manifest
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
%{_datarootdir}/%{ShortName}/scripts/batique.lua
%{_datarootdir}/%{ShortName}/scripts/batique.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/fbm.lua
%{_datarootdir}/%{ShortName}/scripts/fbm.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/perlin.lua
%{_datarootdir}/%{ShortName}/scripts/perlin.lua.manifest
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
%{_datarootdir}/%{ShortName}/scripts/gradient.lua
%{_datarootdir}/%{ShortName}/scripts/gradient.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/heartbeat.lua
%{_datarootdir}/%{ShortName}/scripts/heartbeat.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/impact.lua
%{_datarootdir}/%{ShortName}/scripts/impact.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/multigradient.lua
%{_datarootdir}/%{ShortName}/scripts/multigradient.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/rainbow.lua
%{_datarootdir}/%{ShortName}/scripts/rainbow.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/raindrops.lua
%{_datarootdir}/%{ShortName}/scripts/raindrops.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/shockwave.lua
%{_datarootdir}/%{ShortName}/scripts/shockwave.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/solid.lua
%{_datarootdir}/%{ShortName}/scripts/solid.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/stripes.lua
%{_datarootdir}/%{ShortName}/scripts/stripes.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/sysmon.lua
%{_datarootdir}/%{ShortName}/scripts/sysmon.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/temperature.lua
%{_datarootdir}/%{ShortName}/scripts/temperature.lua.manifest
%{_datarootdir}/%{ShortName}/scripts/water.lua
%{_datarootdir}/%{ShortName}/scripts/water.lua.manifest
#%{_datarootdir}/%{ShortName}/sfx/typewriter1.wav
#%{_datarootdir}/%{ShortName}/sfx/phaser1.wav
#%{_datarootdir}/%{ShortName}/sfx/phaser2.wav
# Web-Frontend
#%{_datarootdir}/%{ShortName}/templates/about.html.tera
#%{_datarootdir}/%{ShortName}/templates/base.html.tera
#%{_datarootdir}/%{ShortName}/templates/detail.html.tera
#%{_datarootdir}/%{ShortName}/templates/documentation.html.tera
#%{_datarootdir}/%{ShortName}/templates/profiles.html.tera
#%{_datarootdir}/%{ShortName}/templates/soundfx.html.tera
#%{_datarootdir}/%{ShortName}/templates/settings.html.tera
#%{_datarootdir}/%{ShortName}/templates/preview.html.tera
#%{_datarootdir}/%{ShortName}/static/css/animate.css
#%{_datarootdir}/%{ShortName}/static/css/style.css
#%{_datarootdir}/%{ShortName}/static/css/themes/eruption/colors.css
#%{_datarootdir}/%{ShortName}/static/css/themes/metal/colors.css
#%{_datarootdir}/%{ShortName}/static/css/styles/tomorrow-night.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap.css.map
#%{_datarootdir}/%{ShortName}/static/css/bootstrap.min.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap.min.css.map
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-grid.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-grid.css.map
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-grid.min.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-grid.min.css.map
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-reboot.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-reboot.css.map
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-reboot.min.css
#%{_datarootdir}/%{ShortName}/static/css/bootstrap-reboot.min.css.map
#%{_datarootdir}/%{ShortName}/static/css/fontawesome.min.css
#%{_datarootdir}/%{ShortName}/static/font/fa-brands-400.eot
#%{_datarootdir}/%{ShortName}/static/font/fa-brands-400.svg
#%{_datarootdir}/%{ShortName}/static/font/fa-brands-400.ttf
#%{_datarootdir}/%{ShortName}/static/font/fa-brands-400.woff
#%{_datarootdir}/%{ShortName}/static/font/fa-brands-400.woff2
#%{_datarootdir}/%{ShortName}/static/font/fa-regular-400.eot
#%{_datarootdir}/%{ShortName}/static/font/fa-regular-400.svg
#%{_datarootdir}/%{ShortName}/static/font/fa-regular-400.ttf
#%{_datarootdir}/%{ShortName}/static/font/fa-regular-400.woff
#%{_datarootdir}/%{ShortName}/static/font/fa-regular-400.woff2
#%{_datarootdir}/%{ShortName}/static/font/fa-solid-900.eot
#%{_datarootdir}/%{ShortName}/static/font/fa-solid-900.svg
#%{_datarootdir}/%{ShortName}/static/font/fa-solid-900.ttf
#%{_datarootdir}/%{ShortName}/static/font/fa-solid-900.woff
#%{_datarootdir}/%{ShortName}/static/font/fa-solid-900.woff2
#%{_datarootdir}/%{ShortName}/static/font/Roboto-Regular.ttf
#%{_datarootdir}/%{ShortName}/static/font/CuteFont-Regular.ttf
#%{_datarootdir}/%{ShortName}/static/font/Roboto-Regular.woff2
#%{_datarootdir}/%{ShortName}/static/font/CuteFont-Regular.woff2
#%{_datarootdir}/%{ShortName}/static/img/bg_direction_nav.png
#%{_datarootdir}/%{ShortName}/static/img/glyphicons-halflings.png
#%{_datarootdir}/%{ShortName}/static/img/bg/bg-1.jpg
#%{_datarootdir}/%{ShortName}/static/img/icons/eruption.png
#%{_datarootdir}/%{ShortName}/static/img/favicon.png
#%{_datarootdir}/%{ShortName}/static/js/animate.js
#%{_datarootdir}/%{ShortName}/static/js/custom.js
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.bundle.js
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.bundle.js.map
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.bundle.min.js
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.bundle.min.js.map
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.js
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.js.map
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.min.js
#%{_datarootdir}/%{ShortName}/static/js/bootstrap.min.js.map
#%{_datarootdir}/%{ShortName}/static/js/jquery.js
#%{_datarootdir}/%{ShortName}/static/js/fontawesome.min.js
#%{_datarootdir}/%{ShortName}/static/js/highlight.pack.js

%changelog
