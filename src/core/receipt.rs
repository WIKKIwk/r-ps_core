use std::fmt;

use crate::scale::Reading;

use super::job::CorePrintJob;
use super::selection::PrintSelection;

pub const MIN_BATCH_QTY_KG: f64 = 0.100;

#[derive(Clone, Debug, PartialEq)]
pub enum PreparePrintJobError {
    MissingWeight,
    MissingItemCode,
    MissingWarehouse,
    GrossQtyTooSmall {
        gross_qty: f64,
    },
    NetQtyTooSmall {
        gross_qty: f64,
        tare_kg: f64,
        net_qty: f64,
    },
}

impl fmt::Display for PreparePrintJobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWeight => write!(f, "scale weight bo'sh"),
            Self::MissingItemCode => write!(f, "material receipt item code bo'sh"),
            Self::MissingWarehouse => write!(f, "material receipt warehouse bo'sh"),
            Self::GrossQtyTooSmall { gross_qty } => write!(
                f,
                "QTY juda kichik: {:.3} kg | min {:.3} kg",
                gross_qty, MIN_BATCH_QTY_KG
            ),
            Self::NetQtyTooSmall {
                gross_qty,
                tare_kg,
                net_qty,
            } => write!(
                f,
                "NETTO juda kichik: brutto {:.3} kg - babina {:.3} kg = {:.3} kg | min {:.3} kg",
                gross_qty, tare_kg, net_qty, MIN_BATCH_QTY_KG
            ),
        }
    }
}

impl std::error::Error for PreparePrintJobError {}

pub fn prepare_print_job(
    reading: &Reading,
    selection: PrintSelection,
    epc: &str,
) -> Result<CorePrintJob, PreparePrintJobError> {
    let selection = selection.normalized();
    if selection.item_code.is_empty() {
        return Err(PreparePrintJobError::MissingItemCode);
    }
    if selection.warehouse.is_empty() {
        return Err(PreparePrintJobError::MissingWarehouse);
    }

    let gross_qty = reading.weight.ok_or(PreparePrintJobError::MissingWeight)?;
    if gross_qty < MIN_BATCH_QTY_KG {
        return Err(PreparePrintJobError::GrossQtyTooSmall { gross_qty });
    }

    let net_qty = selection.net_qty(gross_qty);
    if net_qty < MIN_BATCH_QTY_KG {
        return Err(PreparePrintJobError::NetQtyTooSmall {
            gross_qty,
            tare_kg: selection.tare_kg,
            net_qty,
        });
    }

    Ok(CorePrintJob::from_selection(
        epc,
        net_qty,
        gross_qty,
        &reading.unit,
        selection,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::print::mode::PrintMode;
    use crate::print::printer::PrinterKind;
    use crate::scale::Reading;

    fn selection() -> PrintSelection {
        PrintSelection {
            item_code: " ITEM-1 ".to_string(),
            item_name: " Green Tea ".to_string(),
            warehouse: " Stores - A ".to_string(),
            print_mode: PrintMode::Rfid,
            printer: "zpl".to_string(),
            quantity_source: super::super::selection::QuantitySource::Scale,
            manual_qty_kg: 0.0,
            tare_enabled: false,
            tare_kg: 0.0,
        }
    }

    fn reading(weight: f64, unit: &str) -> Reading {
        Reading::serial("/dev/ttyUSB0", 9600, unit).with_weight(weight, Some(true), "raw")
    }

    #[test]
    fn prepares_print_job_from_stable_scale_reading() {
        let job = prepare_print_job(&reading(2.5, "kg"), selection(), " abc123 ").unwrap();

        assert_eq!(job.epc, "ABC123");
        assert_eq!(job.gross_qty, 2.5);
        assert_eq!(job.net_qty, 2.5);
        assert_eq!(job.unit, "kg");
        assert_eq!(job.item_code, "ITEM-1");
        assert_eq!(job.item_name, "Green Tea");
        assert_eq!(job.printer, Some(PrinterKind::Zebra));
        assert!(!job.tare);
    }

    #[test]
    fn applies_tare_before_building_job_like_gscale() {
        let mut input = selection();
        input.tare_enabled = true;
        input.tare_kg = 0.78;

        let job = prepare_print_job(&reading(2.5, "kg"), input, "EPC-1").unwrap();

        assert_eq!(job.gross_qty, 2.5);
        assert_eq!(job.net_qty, 1.72);
        assert!(job.tare);
        assert_eq!(job.tare_kg, 0.78);
    }

    #[test]
    fn rejects_gross_qty_below_min_like_gscale() {
        let err = prepare_print_job(&reading(0.099, "kg"), selection(), "EPC-1").unwrap_err();

        assert_eq!(
            err,
            PreparePrintJobError::GrossQtyTooSmall { gross_qty: 0.099 }
        );
        assert_eq!(err.to_string(), "QTY juda kichik: 0.099 kg | min 0.100 kg");
    }

    #[test]
    fn rejects_net_qty_below_min_like_gscale() {
        let mut input = selection();
        input.tare_enabled = true;
        input.tare_kg = 0.45;

        let err = prepare_print_job(&reading(0.5, "kg"), input, "EPC-1").unwrap_err();

        assert_eq!(
            err,
            PreparePrintJobError::NetQtyTooSmall {
                gross_qty: 0.5,
                tare_kg: 0.45,
                net_qty: 0.04999999999999999
            }
        );
        assert_eq!(
            err.to_string(),
            "NETTO juda kichik: brutto 0.500 kg - babina 0.450 kg = 0.050 kg | min 0.100 kg"
        );
    }

    #[test]
    fn clamps_negative_net_qty_to_zero_like_gscale_selection() {
        let mut input = selection();
        input.tare_enabled = true;
        input.tare_kg = 0.8;

        let err = prepare_print_job(&reading(0.5, "kg"), input, "EPC-1").unwrap_err();

        assert_eq!(
            err,
            PreparePrintJobError::NetQtyTooSmall {
                gross_qty: 0.5,
                tare_kg: 0.8,
                net_qty: 0.0
            }
        );
    }

    #[test]
    fn falls_back_to_kg_when_scale_unit_is_empty() {
        let job = prepare_print_job(&reading(1.25, ""), selection(), "EPC-1").unwrap();

        assert_eq!(job.unit, "kg");
    }

    #[test]
    fn rejects_missing_weight_and_required_selection_fields() {
        let missing = Reading::serial("/dev/ttyUSB0", 9600, "kg");
        assert_eq!(
            prepare_print_job(&missing, selection(), "EPC-1").unwrap_err(),
            PreparePrintJobError::MissingWeight
        );

        let mut no_item = selection();
        no_item.item_code = " ".to_string();
        assert_eq!(
            prepare_print_job(&reading(1.0, "kg"), no_item, "EPC-1").unwrap_err(),
            PreparePrintJobError::MissingItemCode
        );

        let mut no_warehouse = selection();
        no_warehouse.warehouse = " ".to_string();
        assert_eq!(
            prepare_print_job(&reading(1.0, "kg"), no_warehouse, "EPC-1").unwrap_err(),
            PreparePrintJobError::MissingWarehouse
        );
    }
}
