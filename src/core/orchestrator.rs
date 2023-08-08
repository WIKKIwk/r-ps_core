use std::fmt;

use crate::print::capabilities::{PrinterCapabilities, capabilities_for};
use crate::print::mode::PrintMode;
use crate::print::printer::PrinterKind;
use crate::scale::Reading;

use super::job::CorePrintJob;
use super::receipt::{PreparePrintJobError, prepare_print_job};
use super::selection::PrintSelection;

#[derive(Clone, Debug, PartialEq)]
pub struct CorePrintPlan {
    pub job: CorePrintJob,
    pub printer: PrinterKind,
    pub capabilities: PrinterCapabilities,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CorePrintPlanError {
    PrepareJob(PreparePrintJobError),
    UnsupportedMode {
        printer: PrinterKind,
        mode: PrintMode,
    },
    MissingRequiredFields {
        printer: PrinterKind,
        fields: Vec<&'static str>,
    },
}

impl fmt::Display for CorePrintPlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrepareJob(error) => write!(f, "{error}"),
            Self::UnsupportedMode { printer, mode } => {
                write!(f, "{} does not support {}", printer.as_str(), mode.as_str())
            }
            Self::MissingRequiredFields { printer, fields } => {
                write!(
                    f,
                    "{} print job missing required fields: {}",
                    printer.as_str(),
                    fields.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for CorePrintPlanError {}

pub fn plan_core_print(
    reading: &Reading,
    selection: PrintSelection,
    epc: &str,
) -> Result<CorePrintPlan, CorePrintPlanError> {
    let job = prepare_print_job(reading, selection, epc).map_err(CorePrintPlanError::PrepareJob)?;
    validate_core_print_job(job)
}

pub fn validate_core_print_job(job: CorePrintJob) -> Result<CorePrintPlan, CorePrintPlanError> {
    let printer = job.printer.unwrap_or(PrinterKind::Zebra);
    let capabilities = capabilities_for(printer);

    if !capabilities.supports_mode(job.mode) {
        return Err(CorePrintPlanError::UnsupportedMode {
            printer,
            mode: job.mode,
        });
    }

    let missing = missing_required_fields(&job, &capabilities);
    if !missing.is_empty() {
        return Err(CorePrintPlanError::MissingRequiredFields {
            printer,
            fields: missing,
        });
    }

    Ok(CorePrintPlan {
        job,
        printer,
        capabilities,
    })
}

fn missing_required_fields(
    job: &CorePrintJob,
    capabilities: &PrinterCapabilities,
) -> Vec<&'static str> {
    capabilities
        .required_fields
        .iter()
        .copied()
        .filter(|field| match *field {
            "epc" => job.epc.trim().is_empty(),
            "item_code" => job.item_code.trim().is_empty(),
            "item_name" => job.item_name.trim().is_empty(),
            "weight" => job.net_qty <= 0.0 || job.gross_qty <= 0.0,
            _ => false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::selection::QuantitySource;

    fn selection(printer: &str, mode: PrintMode) -> PrintSelection {
        PrintSelection {
            item_code: "ITEM-1".to_string(),
            item_name: "Green Tea".to_string(),
            warehouse: "Stores - A".to_string(),
            print_mode: mode,
            printer: printer.to_string(),
            quantity_source: QuantitySource::Scale,
            manual_qty_kg: 0.0,
            tare_enabled: false,
            tare_kg: 0.0,
        }
    }

    fn reading(weight: f64) -> Reading {
        Reading::serial("/dev/ttyUSB0", 9600, "kg").with_weight(weight, Some(true), "raw")
    }

    #[test]
    fn plans_zebra_rfid_without_building_hardware_command() {
        let plan =
            plan_core_print(&reading(1.25), selection("zebra", PrintMode::Rfid), "epc-1").unwrap();

        assert_eq!(plan.printer, PrinterKind::Zebra);
        assert_eq!(plan.job.mode, PrintMode::Rfid);
        assert!(plan.capabilities.rfid_epc_write);
    }

    #[test]
    fn plans_godex_label_only_when_capability_allows_it() {
        let plan = plan_core_print(
            &reading(1.25),
            selection("godex", PrintMode::LabelOnly),
            "epc-1",
        )
        .unwrap();

        assert_eq!(plan.printer, PrinterKind::Godex);
        assert_eq!(plan.job.mode, PrintMode::LabelOnly);
        assert!(plan.capabilities.thermal_label);
        assert!(!plan.capabilities.rfid_epc_write);
    }

    #[test]
    fn rejects_godex_rfid_in_core_before_driver_execution() {
        let err = plan_core_print(&reading(1.25), selection("godex", PrintMode::Rfid), "epc-1")
            .unwrap_err();

        assert_eq!(
            err,
            CorePrintPlanError::UnsupportedMode {
                printer: PrinterKind::Godex,
                mode: PrintMode::Rfid,
            }
        );
        assert_eq!(err.to_string(), "godex does not support rfid");
    }

    #[test]
    fn rejects_missing_required_epc_from_capability_contract() {
        let err =
            plan_core_print(&reading(1.25), selection("zebra", PrintMode::Rfid), " ").unwrap_err();

        assert_eq!(
            err,
            CorePrintPlanError::MissingRequiredFields {
                printer: PrinterKind::Zebra,
                fields: vec!["epc"],
            }
        );
    }
}
