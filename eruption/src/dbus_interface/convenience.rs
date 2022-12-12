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

use dbus::arg::{Iter, IterAppend};
use dbus::strings::{Interface as FaceName, Member, Path as PathName};
use dbus::MethodErr;

use super::perms;
use super::{
    Factory, Interface, Method, MethodInfo, MethodResult, Property, PropertyInfo, PropertyResult,
    Tree,
};

pub trait InterfaceAddend {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface;
}

pub trait TreeAdd {
    fn add_path_and_interface(
        self,
        f: &Factory,
        path_name: &PathName<'static>,
        interface_name: &FaceName<'static>,
        to_add: &dyn InterfaceAddend,
    ) -> Self;
}

impl TreeAdd for Tree {
    fn add_path_and_interface(
        self,
        f: &Factory,
        path_name: &PathName<'static>,
        interface_name: &FaceName<'static>,
        to_add: &dyn InterfaceAddend,
    ) -> Self {
        self.add(
            f.object_path(path_name.to_owned(), ())
                .introspectable()
                .add(to_add.add_to_interface(f, f.interface(interface_name.to_owned(), ()))),
        )
    }
}

pub trait FactoryWithPermission {
    fn method_with_permission<H, T>(
        &self,
        t: T,
        permission: perms::Permission,
        handler: H,
    ) -> Method
    where
        H: 'static + Fn(&MethodInfo) -> MethodResult,
        T: Into<Member<'static>>;
}

impl FactoryWithPermission for Factory {
    fn method_with_permission<H, T>(
        &self,
        t: T,
        permission: perms::Permission,
        handler: H,
    ) -> Method
    where
        H: 'static + Fn(&MethodInfo) -> MethodResult,
        T: Into<Member<'static>>,
    {
        self.method(t, (), method_with_permission(permission, handler))
    }
}

fn method_with_permission<H>(
    permission: perms::Permission,
    handler: H,
) -> impl Fn(&MethodInfo) -> MethodResult
where
    H: 'static + Fn(&MethodInfo) -> MethodResult,
{
    move |m: &MethodInfo| {
        if perms::has_permission_cached(permission, &m.msg.sender().unwrap()).unwrap_or(false) {
            handler(m)
        } else {
            Err(MethodErr::failed("Authentication failed"))
        }
    }
}

pub trait PropertyWithPermission {
    fn on_get_with_permission<H>(self, permission: perms::Permission, handler: H) -> Property
    where
        H: 'static + Fn(&mut IterAppend, &PropertyInfo) -> std::result::Result<(), MethodErr>;

    fn on_set_with_permission<H>(self, permission: perms::Permission, handler: H) -> Property
    where
        H: 'static + Fn(&mut Iter, &PropertyInfo) -> std::result::Result<(), MethodErr>;
}

impl PropertyWithPermission for Property {
    fn on_get_with_permission<H>(self, permission: perms::Permission, handler: H) -> Self
    where
        H: 'static + Fn(&mut IterAppend, &PropertyInfo) -> std::result::Result<(), MethodErr>,
    {
        self.on_get(get_with_permission(permission, handler))
    }

    fn on_set_with_permission<H>(self, permission: perms::Permission, handler: H) -> Property
    where
        H: 'static + Fn(&mut Iter, &PropertyInfo) -> std::result::Result<(), MethodErr>,
    {
        self.on_set(set_with_permission(permission, handler))
    }
}

fn get_with_permission<H>(
    permission: perms::Permission,
    handler: H,
) -> impl Fn(&mut IterAppend, &PropertyInfo) -> PropertyResult
where
    H: 'static + Fn(&mut IterAppend, &PropertyInfo) -> PropertyResult,
{
    move |i: &mut IterAppend, m: &PropertyInfo| {
        if perms::has_permission_cached(permission, &m.msg.sender().unwrap()).unwrap_or(false) {
            handler(i, m)
        } else {
            Err(MethodErr::failed("Authentication failed"))
        }
    }
}

fn set_with_permission<H>(
    permission: perms::Permission,
    handler: H,
) -> impl Fn(&mut Iter, &PropertyInfo) -> PropertyResult
where
    H: 'static + Fn(&mut Iter, &PropertyInfo) -> PropertyResult,
{
    move |i: &mut Iter, m: &PropertyInfo| {
        if perms::has_permission_cached(permission, &m.msg.sender().unwrap()).unwrap_or(false) {
            handler(i, m)
        } else {
            Err(MethodErr::failed("Authentication failed"))
        }
    }
}
