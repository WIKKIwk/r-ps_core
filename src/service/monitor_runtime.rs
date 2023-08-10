use std::sync::{Arc, Mutex};

use super::mobile_contract::ServiceIdentity;
use super::monitor_contract::{BatchSnapshot, MonitorPrinter, MonitorResponse};
use crate::print::printer::PrinterKind;
use crate::scale::Reading;

#[derive(Clone, Debug, Default)]
pub struct MonitorRuntimeState {
    last_reading: Arc<Mutex<Option<Reading>>>,
    printer_devices: PrinterDeviceConfig,
}

#[derive(Clone, Debug, Default)]
struct PrinterDeviceConfig {
    zebra_device: Option<String>,
    godex_device: Option<String>,
}

impl MonitorRuntimeState {
    pub fn with_printer_devices(
        zebra_device: Option<String>,
        godex_device: Option<String>,
    ) -> Self {
        Self {
            last_reading: Arc::new(Mutex::new(None)),
            printer_devices: PrinterDeviceConfig {
                zebra_device: normalize_device_path(zebra_device),
                godex_device: normalize_device_path(godex_device),
            },
        }
    }

    pub fn record_reading(&self, reading: Reading) {
        if let Ok(mut last_reading) = self.last_reading.lock() {
            *last_reading = Some(reading);
        }
    }

    pub fn snapshot(
        &self,
        identity: &ServiceIdentity,
        active_printer: PrinterKind,
    ) -> MonitorResponse {
        let reading = self
            .last_reading
            .lock()
            .ok()
            .and_then(|last_reading| last_reading.clone());
        let printer = self.printer_snapshot(active_printer);
        let batch = BatchSnapshot::inactive(active_printer);

        match reading {
            Some(reading) => MonitorResponse::driver_with_scale_and_printer(
                identity,
                active_printer,
                &reading,
                batch,
                printer,
            ),
            None => {
                MonitorResponse::driver_idle_with_printer(identity, active_printer, batch, printer)
            }
        }
    }

    fn printer_snapshot(&self, active_printer: PrinterKind) -> MonitorPrinter {
        let Some(path) = self.device_path(active_printer) else {
            return MonitorPrinter::disconnected(active_printer);
        };
        if std::path::Path::new(&path).exists() {
            MonitorPrinter::connected(active_printer, path)
        } else {
            MonitorPrinter::disconnected_with_error(
                active_printer,
                format!("device not found: {path}"),
            )
        }
    }

    fn device_path(&self, active_printer: PrinterKind) -> Option<String> {
        match active_printer {
            PrinterKind::Zebra => self.printer_devices.zebra_device.clone(),
            PrinterKind::Godex => self.printer_devices.godex_device.clone(),
        }
    }
}

fn normalize_device_path(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::mobile_contract::ServiceIdentity;
    use serde_json::Value;

    #[test]
    fn snapshot_exposes_latest_scale_reading() {
        let runtime = MonitorRuntimeState::default();
        runtime.record_reading(Reading::serial("/dev/tty.sim", 9600, "kg").with_weight(
            1.25,
            Some(true),
            "1.250 kg ST",
        ));

        let identity = ServiceIdentity::new("rp-scale", "rps_1", "RP Scale", "operator");
        let body = serde_json::to_value(runtime.snapshot(&identity, PrinterKind::Zebra)).unwrap();

        assert_eq!(body["state"]["scale"]["source"], "serial");
        assert_eq!(body["state"]["scale"]["port"], "/dev/tty.sim");
        assert_eq!(body["state"]["scale"]["weight"], 1.25);
        assert_eq!(body["state"]["scale"]["stable"], true);
        assert_eq!(body["state"]["printer"]["kind"], "zebra");
    }

    #[test]
    fn empty_snapshot_is_mobile_safe() {
        let runtime = MonitorRuntimeState::default();
        let identity = ServiceIdentity::new("rp-scale", "rps_1", "RP Scale", "operator");
        let body = serde_json::to_value(runtime.snapshot(&identity, PrinterKind::Godex)).unwrap();

        assert_eq!(body["state"]["scale"]["weight"], Value::Null);
        assert_eq!(body["state"]["batch"]["printer"], "godex");
        assert_eq!(body["state"]["batch"]["print_mode"], "label");
    }

    #[test]
    fn snapshot_reports_configured_godex_device_as_connected() {
        let path =
            std::env::temp_dir().join(format!("rp-scale-godex-monitor-{}", std::process::id()));
        std::fs::File::create(&path).unwrap();
        let runtime = MonitorRuntimeState::with_printer_devices(
            None,
            Some(path.to_string_lossy().to_string()),
        );
        let identity = ServiceIdentity::new("rp-scale", "rps_1", "RP Scale", "operator");
        let body = serde_json::to_value(runtime.snapshot(&identity, PrinterKind::Godex)).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(body["state"]["printer"]["ok"], true);
        assert_eq!(body["state"]["printer"]["connected"], true);
        assert_eq!(body["state"]["printer"]["kind"], "godex");
        assert_eq!(body["state"]["printer"]["label"], "ulangan");
        assert_eq!(
            body["state"]["printer"]["device_paths"][0],
            path.to_string_lossy().as_ref()
        );
        assert_eq!(body["printer"], body["state"]["printer"]);
    }

    #[test]
    fn snapshot_keeps_batch_inactive_until_split_contract_exists() {
        let runtime = MonitorRuntimeState::default();
        let identity = ServiceIdentity::new("rp-scale", "rps_1", "RP Scale", "operator");
        let body = serde_json::to_value(runtime.snapshot(&identity, PrinterKind::Zebra)).unwrap();

        assert_eq!(body["state"]["batch"]["active"], false);
        assert_eq!(body["state"]["batch"]["item_code"], "");
        assert_eq!(body["state"]["batch"]["warehouse"], "");
    }
}
