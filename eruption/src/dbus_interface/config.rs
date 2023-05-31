/*  SPDX-License-Identifier: GPL-3.0-or-later  */

/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use colorgrad::Color;
use dbus::arg::{Iter, IterAppend};
use dbus_tree::{Access, EmitsChangedSignal, MethodErr, MethodResult, Signal};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[cfg(not(target_os = "windows"))]
use crate::plugins::audio;

use crate::{color_scheme::ColorScheme, script};

use super::{
    convenience::FactoryWithPermission, convenience::InterfaceAddend,
    convenience::PropertyWithPermission, perms::Permission, Factory, Interface, MethodInfo,
    Property,
};

pub struct ConfigInterface {
    pub brightness_changed_signal: Arc<Signal<()>>,
    enable_sfx_property: Arc<Property>,
    brightness_property: Arc<Property>,
}

impl ConfigInterface {
    pub fn new(f: &Factory) -> Self {
        let brightness_changed_signal = Arc::new(
            f.signal("BrightnessChanged", ())
                .sarg::<i64, _>("brightness"),
        );

        let enable_sfx_property = Arc::new(
            f.property::<bool, _>("EnableSfx", ())
                .access(Access::ReadWrite)
                .emits_changed(EmitsChangedSignal::True)
                .auto_emit_on_set(true)
                .on_get_with_permission(Permission::Monitor, get_enable_sfx)
                .on_set_with_permission(Permission::Settings, set_enable_sfx),
        );

        let brightness_property = Arc::new(
            f.property::<i64, _>("Brightness", ())
                .access(Access::ReadWrite)
                .emits_changed(EmitsChangedSignal::True)
                .auto_emit_on_set(true)
                .on_get_with_permission(Permission::Monitor, get_brightness)
                .on_set_with_permission(Permission::Settings, set_brightness),
        );

        Self {
            brightness_changed_signal,
            enable_sfx_property,
            brightness_property,
        }
    }
}

impl InterfaceAddend for ConfigInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        interface
            .add_s(self.brightness_changed_signal.clone())
            .add_p(self.enable_sfx_property.clone())
            .add_p(self.brightness_property.clone())
            .add_m(
                f.method_with_permission("WriteFile", Permission::Manage, write_file)
                    .inarg::<String, _>("filename")
                    .inarg::<String, _>("data")
                    .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission("Ping", Permission::Monitor, ping)
                    .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission("PingPrivileged", Permission::Manage, ping)
                    .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission("GetColorSchemes", Permission::Monitor, get_color_schemes)
                    .outarg::<Vec<String>, _>("color_schemes"),
            )
            .add_m(
                f.method_with_permission("GetColorScheme", Permission::Settings, get_color_scheme)
                    .inarg::<String, _>("name")
                    .outarg::<Vec<u8>, _>("data"),
            )
            .add_m(
                f.method_with_permission("SetColorScheme", Permission::Settings, set_color_scheme)
                    .inarg::<String, _>("name")
                    .inarg::<Vec<u8>, _>("data")
                    .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission(
                    "RemoveColorScheme",
                    Permission::Settings,
                    remove_color_scheme,
                )
                .inarg::<String, _>("name")
                .outarg::<bool, _>("status"),
            )
    }
}

fn get_enable_sfx(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    #[cfg(not(target_os = "windows"))]
    i.append(audio::ENABLE_SFX.load(Ordering::SeqCst));

    Ok(())
}

fn set_enable_sfx(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    #[cfg(not(target_os = "windows"))]
    audio::ENABLE_SFX.store(i.read::<bool>()?, Ordering::SeqCst);

    Ok(())
}

fn get_brightness(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    let result = crate::BRIGHTNESS.load(Ordering::SeqCst) as i64;
    i.append(result);
    Ok(())
}

fn set_brightness(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    crate::BRIGHTNESS.store(i.read::<i64>()? as isize, Ordering::SeqCst);
    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

// NOTE: possible security issue when eruption is run as root
fn write_file(m: &MethodInfo) -> MethodResult {
    let (filename, data): (String, String) = m.msg.read2()?;

    crate::util::write_file(&PathBuf::from(filename), &data)
        .map_err(|e| MethodErr::failed(&format!("Error writing file: {e}")))?;

    Ok(vec![m.msg.method_return().append1(true)])
}

fn ping(m: &MethodInfo) -> MethodResult {
    Ok(vec![m.msg.method_return().append1(true)])
}

fn get_color_schemes(m: &MethodInfo) -> MethodResult {
    let color_schemes: Vec<String> = crate::NAMED_COLOR_SCHEMES.read().keys().cloned().collect();
    Ok(vec![m.msg.method_return().append1(color_schemes)])
}

fn get_color_scheme(m: &MethodInfo) -> MethodResult {
    let name: String = m.msg.read1()?;

    let color_schemes = crate::NAMED_COLOR_SCHEMES.read();
    if let Some(color_scheme) = color_schemes.get(&name) {
        let colors = color_scheme
            .colors
            .iter()
            .flat_map(|e| {
                vec![
                    (255.0 * e.r).round() as u8,
                    (255.0 * e.g).round() as u8,
                    (255.0 * e.b).round() as u8,
                    (255.0 * e.a).round() as u8,
                ]
            })
            .collect::<Vec<u8>>();

        Ok(vec![m.msg.method_return().append1(colors)])
    } else {
        Err(MethodErr::failed("Invalid identifier name"))
    }
}

fn set_color_scheme(m: &MethodInfo) -> MethodResult {
    let (name, data): (String, Vec<u8>) = m.msg.read2()?;

    if name.chars().take(1).all(char::is_numeric)
        || !name
            .chars()
            .all(|c| c == '_' || char::is_ascii_alphanumeric(&c))
    {
        Err(MethodErr::failed("Invalid identifier name"))
    } else {
        let mut color_schemes = crate::NAMED_COLOR_SCHEMES.write();
        let mut colors = Vec::new();

        for chunk in data.chunks(4) {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            let a = chunk[3];

            let color = Color::from_linear_rgba8(r, g, b, a);

            colors.push(color);
        }

        color_schemes.insert(name, ColorScheme { colors });

        crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);

        Ok(vec![m.msg.method_return().append1(true)])
    }
}

fn remove_color_scheme(m: &MethodInfo) -> MethodResult {
    let name: String = m.msg.read1()?;

    let s = crate::NAMED_COLOR_SCHEMES.write().remove(&name).is_some();

    if s {
        crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
    }

    Ok(vec![m.msg.method_return().append1(s)])
}
