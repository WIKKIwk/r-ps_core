#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ScaleTransport {
    Serial,
    Usb,
    Wifi,
    Bluetooth,
    VendorSdk,
    Simulated,
}

impl ScaleTransport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Serial => "serial",
            Self::Usb => "usb",
            Self::Wifi => "wifi",
            Self::Bluetooth => "bluetooth",
            Self::VendorSdk => "vendor_sdk",
            Self::Simulated => "simulated",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScaleCapabilities {
    pub driver_id: String,
    pub display_name: String,
    pub transport: ScaleTransport,
    pub realtime_weight: bool,
    pub stability_flag: bool,
    pub raw_diagnostics: bool,
    pub default_unit: String,
    pub connection: String,
}

impl ScaleCapabilities {
    pub fn serial(port: &str, baud: u32, default_unit: &str) -> Self {
        Self {
            driver_id: "serial-scale".to_string(),
            display_name: "Serial Scale".to_string(),
            transport: ScaleTransport::Serial,
            realtime_weight: true,
            stability_flag: true,
            raw_diagnostics: true,
            default_unit: normalize_unit(default_unit),
            connection: format!("{}@{}", port.trim(), baud),
        }
    }
}

fn normalize_unit(unit: &str) -> String {
    let unit = unit.trim().to_ascii_lowercase();
    if unit.is_empty() {
        "kg".to_string()
    } else {
        unit
    }
}

#[cfg(test)]
mod tests {
    use super::{ScaleCapabilities, ScaleTransport};

    #[test]
    fn serial_capabilities_describe_current_driver() {
        let caps = ScaleCapabilities::serial("/dev/ttyUSB0", 9600, "");

        assert_eq!(caps.driver_id, "serial-scale");
        assert_eq!(caps.display_name, "Serial Scale");
        assert_eq!(caps.transport, ScaleTransport::Serial);
        assert_eq!(caps.transport.as_str(), "serial");
        assert!(caps.realtime_weight);
        assert!(caps.stability_flag);
        assert!(caps.raw_diagnostics);
        assert_eq!(caps.default_unit, "kg");
        assert_eq!(caps.connection, "/dev/ttyUSB0@9600");
    }
}
