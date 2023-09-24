// This code was autogenerated with `dbus-codegen-rust --system-bus --prop-newtype --destination org.eruption --path /org/eruption/canvas`, see https://github.com/diwic/dbus-rs
use dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;

pub trait OrgEruptionCanvas {
    fn get_devices_zone_allocations(
        &self,
    ) -> Result<Vec<(u64, (i32, i32, i32, i32, bool))>, dbus::Error>;
    fn set_device_zone_allocation(
        &self,
        device: u64,
        zone: (i32, i32, i32, i32, bool),
    ) -> Result<(), dbus::Error>;
    fn set_devices_zone_allocations(
        &self,
        zones: Vec<(u64, (i32, i32, i32, i32, bool))>,
    ) -> Result<(), dbus::Error>;
    fn hue(&self) -> Result<f64, dbus::Error>;
    fn set_hue(&self, value: f64) -> Result<(), dbus::Error>;
    fn lightness(&self) -> Result<f64, dbus::Error>;
    fn set_lightness(&self, value: f64) -> Result<(), dbus::Error>;
    fn saturation(&self) -> Result<f64, dbus::Error>;
    fn set_saturation(&self, value: f64) -> Result<(), dbus::Error>;
}

#[derive(Debug)]
pub struct OrgEruptionCanvasHueChanged {
    pub hue: f64,
}

impl arg::AppendAll for OrgEruptionCanvasHueChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.hue, i);
    }
}

impl arg::ReadAll for OrgEruptionCanvasHueChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionCanvasHueChanged { hue: i.read()? })
    }
}

impl dbus::message::SignalArgs for OrgEruptionCanvasHueChanged {
    const NAME: &'static str = "HueChanged";
    const INTERFACE: &'static str = "org.eruption.Canvas";
}

#[derive(Debug)]
pub struct OrgEruptionCanvasLightnessChanged {
    pub lightness: f64,
}

impl arg::AppendAll for OrgEruptionCanvasLightnessChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.lightness, i);
    }
}

impl arg::ReadAll for OrgEruptionCanvasLightnessChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionCanvasLightnessChanged {
            lightness: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgEruptionCanvasLightnessChanged {
    const NAME: &'static str = "LightnessChanged";
    const INTERFACE: &'static str = "org.eruption.Canvas";
}

#[derive(Debug)]
pub struct OrgEruptionCanvasSaturationChanged {
    pub saturation: f64,
}

impl arg::AppendAll for OrgEruptionCanvasSaturationChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.saturation, i);
    }
}

impl arg::ReadAll for OrgEruptionCanvasSaturationChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(OrgEruptionCanvasSaturationChanged {
            saturation: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for OrgEruptionCanvasSaturationChanged {
    const NAME: &'static str = "SaturationChanged";
    const INTERFACE: &'static str = "org.eruption.Canvas";
}

pub const ORG_ERUPTION_CANVAS_NAME: &str = "org.eruption.Canvas";

#[derive(Copy, Clone, Debug)]
pub struct OrgEruptionCanvasProperties<'a>(pub &'a arg::PropMap);

impl<'a> OrgEruptionCanvasProperties<'a> {
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {
        interfaces.get("org.eruption.Canvas").map(Self)
    }

    pub fn hue(&self) -> Option<f64> {
        arg::prop_cast(self.0, "Hue").copied()
    }

    pub fn lightness(&self) -> Option<f64> {
        arg::prop_cast(self.0, "Lightness").copied()
    }

    pub fn saturation(&self) -> Option<f64> {
        arg::prop_cast(self.0, "Saturation").copied()
    }
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> OrgEruptionCanvas
    for blocking::Proxy<'a, C>
{
    fn get_devices_zone_allocations(
        &self,
    ) -> Result<Vec<(u64, (i32, i32, i32, i32, bool))>, dbus::Error> {
        self.method_call("org.eruption.Canvas", "GetDevicesZoneAllocations", ())
            .map(|r: (Vec<(u64, (i32, i32, i32, i32, bool))>,)| r.0)
    }

    fn set_device_zone_allocation(
        &self,
        device: u64,
        zone: (i32, i32, i32, i32, bool),
    ) -> Result<(), dbus::Error> {
        self.method_call(
            "org.eruption.Canvas",
            "SetDeviceZoneAllocation",
            (device, zone),
        )
    }

    fn set_devices_zone_allocations(
        &self,
        zones: Vec<(u64, (i32, i32, i32, i32, bool))>,
    ) -> Result<(), dbus::Error> {
        self.method_call("org.eruption.Canvas", "SetDevicesZoneAllocations", (zones,))
    }

    fn hue(&self) -> Result<f64, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Canvas",
            "Hue",
        )
    }

    fn lightness(&self) -> Result<f64, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Canvas",
            "Lightness",
        )
    }

    fn saturation(&self) -> Result<f64, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.eruption.Canvas",
            "Saturation",
        )
    }

    fn set_hue(&self, value: f64) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.eruption.Canvas",
            "Hue",
            value,
        )
    }

    fn set_lightness(&self, value: f64) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.eruption.Canvas",
            "Lightness",
            value,
        )
    }

    fn set_saturation(&self, value: f64) -> Result<(), dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::set(
            self,
            "org.eruption.Canvas",
            "Saturation",
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
