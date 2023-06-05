// This code was autogenerated with `dbus-codegen-rust --system-bus --prop-newtype --destination org.eruption --path /org/eruption/devices`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgEruptionDevice {
    fn get_device_config(&self, device: u64, param: &str) -> Result<String, dbus::Error>;
    fn get_device_status(&self, device: u64) -> Result<String, dbus::Error>;
    fn get_managed_devices(
        &self,
    ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error>;
    fn is_device_enabled(&self, device: u64) -> Result<bool, dbus::Error>;
    fn set_device_config(&self, device: u64, param: &str, value: &str)
        -> Result<bool, dbus::Error>;
    fn set_device_enabled(&self, device: u64, enabled: bool) -> Result<bool, dbus::Error>;
    fn device_status(&self) -> Result<String, dbus::Error>;
}

#[derive(Debug)]
pub struct OrgEruptionDeviceDeviceHotplug {
    pub device_info: (u16, u16, bool),
}

impl arg::AppendAll for OrgEruptionDeviceDeviceHotplug {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.device_info, i);
    }
}

impl arg::ReadAll for OrgEruptionDeviceDeviceHotplug {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionDeviceDeviceHotplug {
            device_info: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgEruptionDeviceDeviceHotplug {
    const NAME: &'static str = "DeviceHotplug";
    const INTERFACE: &'static str = "org.eruption.Device";
}

#[derive(Debug)]
pub struct OrgEruptionDeviceDeviceStatusChanged {
    pub status: String,
}

impl arg::AppendAll for OrgEruptionDeviceDeviceStatusChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.status, i);
    }
}

impl arg::ReadAll for OrgEruptionDeviceDeviceStatusChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionDeviceDeviceStatusChanged { status: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgEruptionDeviceDeviceStatusChanged {
    const NAME: &'static str = "DeviceStatusChanged";
    const INTERFACE: &'static str = "org.eruption.Device";
}

pub const ORG_ERUPTION_DEVICE_NAME: &str = "org.eruption.Device";

#[derive(Copy, Clone, Debug)]
pub struct OrgEruptionDeviceProperties<'a>(pub &'a arg::PropMap);

impl<'a> OrgEruptionDeviceProperties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.eruption.Device").map(Self)
    }

    pub fn device_status(&self) -> Option<&String> {
        arg::prop_cast(self.0, "DeviceStatus")
    }
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionDevice
    for blocking::Proxy<'a, C>
{
    fn get_device_config(&self, device: u64, param: &str) -> Result<String, dbus::Error> {
        self.method_call("org.eruption.Device", "GetDeviceConfig", (device, param))
            .map(|r: (String,)| r.0)
    }

    fn get_device_status(&self, device: u64) -> Result<String, dbus::Error> {
        self.method_call("org.eruption.Device", "GetDeviceStatus", (device,))
            .map(|r: (String,)| r.0)
    }

    fn get_managed_devices(
        &self,
    ) -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), dbus::Error> {
        self.method_call("org.eruption.Device", "GetManagedDevices", ())
            .map(|r: ((Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>),)| r.0)
    }

    fn is_device_enabled(&self, device: u64) -> Result<bool, dbus::Error> {
        self.method_call("org.eruption.Device", "IsDeviceEnabled", (device,))
            .map(|r: (bool,)| r.0)
    }

    fn set_device_config(
        &self,
        device: u64,
        param: &str,
        value: &str,
    ) -> Result<bool, dbus::Error> {
        self.method_call(
            "org.eruption.Device",
            "SetDeviceConfig",
            (device, param, value),
        )
        .map(|r: (bool,)| r.0)
    }

    fn set_device_enabled(&self, device: u64, enabled: bool) -> Result<bool, dbus::Error> {
        self.method_call("org.eruption.Device", "SetDeviceEnabled", (device, enabled))
            .map(|r: (bool,)| r.0)
    }

    fn device_status(&self) -> Result<String, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Device",
            "DeviceStatus",
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
