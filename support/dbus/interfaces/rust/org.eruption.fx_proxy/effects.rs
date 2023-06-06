// This code was autogenerated with `dbus-codegen-rust --prop-newtype --destination org.eruption.fx_proxy --path /org/eruption/fx_proxy/effects`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgEruptionFxProxyEffects {
    fn disable_ambient_effect(&self) -> Result<(), dbus::Error>;
    fn enable_ambient_effect(&self) -> Result<(), dbus::Error>;
    fn ambient_effect(&self) -> Result<bool, dbus::Error>;
    fn set_ambient_effect(&self, value: bool) -> Result<(), dbus::Error>;
}

#[derive(Debug)]
pub struct OrgEruptionFxProxyEffectsStatusChanged {
    pub event: String,
}

impl arg::AppendAll for OrgEruptionFxProxyEffectsStatusChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.event, i);
    }
}

impl arg::ReadAll for OrgEruptionFxProxyEffectsStatusChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionFxProxyEffectsStatusChanged { event: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgEruptionFxProxyEffectsStatusChanged {
    const NAME: &'static str = "StatusChanged";
    const INTERFACE: &'static str = "org.eruption.fx_proxy.Effects";
}

pub const ORG_ERUPTION_FX_PROXY_EFFECTS_NAME: &str = "org.eruption.fx_proxy.Effects";

#[derive(Copy, Clone, Debug)]
pub struct OrgEruptionFxProxyEffectsProperties<'a>(pub &'a arg::PropMap);

impl<'a> OrgEruptionFxProxyEffectsProperties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.eruption.fx_proxy.Effects").map(Self)
    }

    pub fn ambient_effect(&self) -> Option<bool> {
        arg::prop_cast(self.0, "AmbientEffect").copied()
    }
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionFxProxyEffects
    for blocking::Proxy<'a, C>
{
    fn disable_ambient_effect(&self) -> Result<(), dbus::Error> {
        self.method_call("org.eruption.fx_proxy.Effects", "DisableAmbientEffect", ())
    }

    fn enable_ambient_effect(&self) -> Result<(), dbus::Error> {
        self.method_call("org.eruption.fx_proxy.Effects", "EnableAmbientEffect", ())
    }

    fn ambient_effect(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            &self,
            "org.eruption.fx_proxy.Effects",
            "AmbientEffect",
        )
    }

    fn set_ambient_effect(&self, value: bool) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            &self,
            "org.eruption.fx_proxy.Effects",
            "AmbientEffect",
            value,
        )
    }
}

pub trait OrgFreedesktopDBusIntrospectable {
    fn introspect(&self) -> Result<String, dbus::Error>;
}

pub const ORG_FREEDESKTOP_DBUS_INTROSPECTABLE_NAME: &str = "org.freedesktop.DBus.Introspectable";

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>>
    OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
{
    fn introspect(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
            .and_then(|r: (String,)| Ok(r.0))
    }
}

pub trait OrgFreedesktopDBusProperties {
    fn get(
        &self,
        interface_name: &str,
        property_name: &str,
    ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
    fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error>;
    fn set(
        &self,
        interface_name: &str,
        property_name: &str,
        value: arg::Variant<Box<dyn arg::RefArg>>,
    ) -> Result<(), dbus::Error>;
}

#[derive(Debug)]
pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
    pub interface_name: String,
    pub changed_properties: arg::PropMap,
    pub invalidated_properties: Vec<String>,
}

impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.interface_name, i);
        arg::RefArg::append(&self.changed_properties, i);
        arg::RefArg::append(&self.invalidated_properties, i);
    }
}

impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
            interface_name: i.read()?,
            changed_properties: i.read()?,
            invalidated_properties: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
    const NAME: &'static str = "PropertiesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
}

pub const ORG_FREEDESKTOP_DBUS_PROPERTIES_NAME: &str = "org.freedesktop.DBus.Properties";

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgFreedesktopDBusProperties
    for blocking::Proxy<'a, C>
{
    fn get(
        &self,
        interface_name: &str,
        property_name: &str,
    ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "Get",
            (interface_name, property_name),
        )
        .and_then(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| Ok(r.0))
    }

    fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "GetAll",
            (interface_name,),
        )
        .and_then(|r: (arg::PropMap,)| Ok(r.0))
    }

    fn set(
        &self,
        interface_name: &str,
        property_name: &str,
        value: arg::Variant<Box<dyn arg::RefArg>>,
    ) -> Result<(), dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "Set",
            (interface_name, property_name, value),
        )
    }
}
