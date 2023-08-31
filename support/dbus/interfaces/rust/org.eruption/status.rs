// This code was autogenerated with `dbus-codegen-rust --system-bus --prop-newtype --destination org.eruption --path /org/eruption/status`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgEruptionStatus {
    fn get_led_colors(&self) -> Result<Vec<(u8, u8, u8, u8)>, dbus::Error>;
    fn get_managed_devices(
        &self,
    ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error>;
    fn running(&self) -> Result<bool, dbus::Error>;
}

pub const ORG_ERUPTION_STATUS_NAME: &str = "org.eruption.Status";

#[derive(Copy, Clone, Debug)]
pub struct OrgEruptionStatusProperties<'a>(pub &'a arg::PropMap);

impl<'a> OrgEruptionStatusProperties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.eruption.Status").map(Self)
    }

    pub fn running(&self) -> Option<bool> {
        arg::prop_cast(self.0, "Running").copied()
    }
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionStatus
    for blocking::Proxy<'a, C>
{
    fn get_led_colors(&self) -> Result<Vec<(u8, u8, u8, u8)>, dbus::Error> {
        self.method_call("org.eruption.Status", "GetLedColors", ())
            .and_then(|r: (Vec<(u8, u8, u8, u8)>,)| Ok(r.0))
    }

    fn get_managed_devices(
        &self,
    ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error> {
        self.method_call("org.eruption.Status", "GetManagedDevices", ())
            .and_then(|r: ((Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>),)| Ok(r.0))
    }

    fn running(&self) -> Result<bool, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Status",
            "Running",
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
