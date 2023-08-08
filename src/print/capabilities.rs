use super::mode::PrintMode;
use super::printer::PrinterKind;

#[derive(Clone, Debug, PartialEq)]
pub struct PrinterCapabilities {
    pub id: &'static str,
    pub name: &'static str,
    pub thermal_label: bool,
    pub rfid_epc_write: bool,
    pub barcode: bool,
    pub qr: bool,
    pub verify_after_print: bool,
    pub required_fields: &'static [&'static str],
    pub unsupported_modes: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrinterCapabilityFlags {
    pub thermal_label: bool,
    pub rfid_epc_write: bool,
    pub barcode: bool,
    pub qr: bool,
    pub verify_after_print: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActivePrinterManifest {
    pub id: &'static str,
    pub name: &'static str,
    pub capabilities: PrinterCapabilityFlags,
    pub required_fields: &'static [&'static str],
    pub unsupported_modes: &'static [&'static str],
}

impl PrinterCapabilities {
    pub fn supports_mode(&self, mode: PrintMode) -> bool {
        match mode {
            PrintMode::Rfid => self.rfid_epc_write,
            PrintMode::LabelOnly => self.thermal_label,
        }
    }

    pub fn flags(&self) -> PrinterCapabilityFlags {
        PrinterCapabilityFlags {
            thermal_label: self.thermal_label,
            rfid_epc_write: self.rfid_epc_write,
            barcode: self.barcode,
            qr: self.qr,
            verify_after_print: self.verify_after_print,
        }
    }

    pub fn manifest(&self) -> ActivePrinterManifest {
        ActivePrinterManifest {
            id: self.id,
            name: self.name,
            capabilities: self.flags(),
            required_fields: self.required_fields,
            unsupported_modes: self.unsupported_modes,
        }
    }
}

pub fn capabilities_for(kind: PrinterKind) -> PrinterCapabilities {
    match kind {
        PrinterKind::Zebra => zebra_capabilities(),
        PrinterKind::Godex => godex_capabilities(),
    }
}

pub fn manifest_for(kind: PrinterKind) -> ActivePrinterManifest {
    capabilities_for(kind).manifest()
}

pub fn zebra_capabilities() -> PrinterCapabilities {
    PrinterCapabilities {
        id: "zebra",
        name: "Zebra RFID",
        thermal_label: true,
        rfid_epc_write: true,
        barcode: true,
        qr: false,
        verify_after_print: true,
        required_fields: &["epc", "item_name", "weight"],
        unsupported_modes: &[],
    }
}

pub fn godex_capabilities() -> PrinterCapabilities {
    PrinterCapabilities {
        id: "godex",
        name: "GoDEX G500",
        thermal_label: true,
        rfid_epc_write: false,
        barcode: true,
        qr: true,
        verify_after_print: false,
        required_fields: &["epc", "item_name", "weight"],
        unsupported_modes: &["rfid_epc_write"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_zebra_capabilities() {
        let caps = capabilities_for(PrinterKind::Zebra);

        assert!(caps.supports_mode(PrintMode::Rfid));
        assert!(caps.supports_mode(PrintMode::LabelOnly));
        assert!(caps.rfid_epc_write);
        assert!(caps.verify_after_print);
    }

    #[test]
    fn exposes_godex_capabilities_and_blocks_rfid() {
        let caps = capabilities_for(PrinterKind::Godex);

        assert!(!caps.supports_mode(PrintMode::Rfid));
        assert!(caps.supports_mode(PrintMode::LabelOnly));
        assert_eq!(caps.unsupported_modes, &["rfid_epc_write"]);
        assert!(caps.qr);
        assert!(caps.barcode);
    }

    #[test]
    fn exports_mobile_safe_manifest_without_driver_details() {
        let manifest = manifest_for(PrinterKind::Godex);

        assert_eq!(manifest.id, "godex");
        assert_eq!(manifest.name, "GoDEX G500");
        assert!(manifest.capabilities.thermal_label);
        assert!(!manifest.capabilities.rfid_epc_write);
        assert_eq!(manifest.required_fields, &["epc", "item_name", "weight"]);
        assert_eq!(manifest.unsupported_modes, &["rfid_epc_write"]);
    }
}
