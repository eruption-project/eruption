// This code was autogenerated with `dbus-codegen-rust --system-bus --prop-newtype --destination org.eruption --path /org/eruption/slot`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgEruptionSlot {
    fn get_slot_profiles(&self) -> Result<Vec<String>, dbus::Error>;
    fn switch_slot(&self, slot: u64) -> Result<bool, dbus::Error>;
    fn active_slot(&self) -> Result<u64, dbus::Error>;
    fn slot_names(&self) -> Result<Vec<String>, dbus::Error>;
    fn set_slot_names(&self, value: Vec<String>) -> Result<(), dbus::Error>;
}

#[derive(Debug)]
pub struct OrgEruptionSlotActiveSlotChanged {
    pub slot: u64,
}

impl arg::AppendAll for OrgEruptionSlotActiveSlotChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.slot, i);
    }
}

impl arg::ReadAll for OrgEruptionSlotActiveSlotChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionSlotActiveSlotChanged { slot: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgEruptionSlotActiveSlotChanged {
    const NAME: &'static str = "ActiveSlotChanged";
    const INTERFACE: &'static str = "org.eruption.Slot";
}

pub const ORG_ERUPTION_SLOT_NAME: &str = "org.eruption.Slot";

#[derive(Copy, Clone, Debug)]
pub struct OrgEruptionSlotProperties<'a>(pub &'a arg::PropMap);

impl<'a> OrgEruptionSlotProperties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.eruption.Slot").map(Self)
    }

    pub fn active_slot(&self) -> Option<u64> {
        arg::prop_cast(self.0, "ActiveSlot").copied()
    }

    pub fn slot_names(&self) -> Option<&Vec<String>> {
        arg::prop_cast(self.0, "SlotNames")
    }
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionSlot
    for blocking::Proxy<'a, C>
{
    fn get_slot_profiles(&self) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.eruption.Slot", "GetSlotProfiles", ())
            .map(|r: (Vec<String>,)| r.0)
    }

    fn switch_slot(&self, slot: u64) -> Result<bool, dbus::Error> {
        self.method_call("org.eruption.Slot", "SwitchSlot", (slot,))
            .map(|r: (bool,)| r.0)
    }

    fn active_slot(&self) -> Result<u64, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Slot",
            "ActiveSlot",
        )
    }

    fn slot_names(&self) -> Result<Vec<String>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Slot",
            "SlotNames",
        )
    }

    fn set_slot_names(&self, value: Vec<String>) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.eruption.Slot",
            "SlotNames",
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
            .map(|r: (String,)| r.0)
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
        .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
    }

    fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus.Properties",
            "GetAll",
            (interface_name,),
        )
        .map(|r: (arg::PropMap,)| r.0)
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