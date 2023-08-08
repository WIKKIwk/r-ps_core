use std::fmt;

use crate::core::{CorePrintPlan, CorePrintPlanError, PrintSelection, plan_core_print};
use crate::print::adapter::{PrintAdapterError, PrintCommand, build_print_command};
use crate::scale::Reading;

#[derive(Clone, Debug, PartialEq)]
pub struct PrintPipelineResult {
    pub plan: CorePrintPlan,
    pub command: PrintCommand,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrintPipelineError {
    Core(CorePrintPlanError),
    Adapter(PrintAdapterError),
}

impl fmt::Display for PrintPipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(error) => write!(f, "{error}"),
            Self::Adapter(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PrintPipelineError {}

pub fn prepare_print_command(
    reading: &Reading,
    selection: PrintSelection,
    epc: &str,
) -> Result<PrintPipelineResult, PrintPipelineError> {
    let plan = plan_core_print(reading, selection, epc).map_err(PrintPipelineError::Core)?;
    let command = build_print_command(plan.clone()).map_err(PrintPipelineError::Adapter)?;

    Ok(PrintPipelineResult { plan, command })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::QuantitySource;
    use crate::print::adapter::PrintCommand;
    use crate::print::mode::PrintMode;
    use crate::print::printer::PrinterKind;
    use crate::scale::Reading;

    fn selection(printer: &str, mode: PrintMode) -> PrintSelection {
        PrintSelection {
            item_code: "ITEM-1".to_string(),
            item_name: "Green Tea".to_string(),
            warehouse: "Stores - A".to_string(),
            print_mode: mode,
            printer: printer.to_string(),
            quantity_source: QuantitySource::Scale,
            manual_qty_kg: 0.0,
            tare_enabled: true,
            tare_kg: 0.78,
        }
    }

    fn reading(weight: f64) -> Reading {
        Reading::serial("/dev/ttyUSB0", 9600, "kg").with_weight(weight, Some(true), "raw")
    }

    #[test]
    fn prepares_zebra_rfid_command_from_scale_reading() {
        let result = prepare_print_command(
            &reading(2.5),
            selection("zebra", PrintMode::Rfid),
            "3034257BF7194E406994036B",
        )
        .unwrap();

        assert_eq!(result.plan.printer, PrinterKind::Zebra);
        let PrintCommand::ZebraZpl(command) = result.command else {
            panic!("expected zebra zpl command");
        };
        assert!(command.contains("^RFW,H,,,A^FD3034257BF7194E406994036B^FS"));
        assert!(command.contains("^FDNETTO: 1.7 kg^FS"));
        assert!(command.contains("^FDBRUTTO: 2.5 kg^FS"));
    }

    #[test]
    fn prepares_godex_pack_render_from_scale_reading() {
        let result = prepare_print_command(
            &reading(2.5),
            selection("godex", PrintMode::LabelOnly),
            "3034257BF7194E406994036B",
        )
        .unwrap();

        assert_eq!(result.plan.printer, PrinterKind::Godex);
        let PrintCommand::GodexPack(render) = result.command else {
            panic!("expected godex pack render");
        };
        assert_eq!(
            render.qr_payload,
            "https://scan.wspace.sbs/L/ACCORD/GREEN+TEA/1.7/2.5/3034257BF7194E406994036B"
        );
        assert_eq!(render.commands[11], "Y0,0,TEXTLBL");
        assert_eq!(render.commands[13], "Y224,224,QRLBL");
    }

    #[test]
    fn rejects_godex_rfid_before_adapter_runs() {
        let err = prepare_print_command(
            &reading(2.5),
            selection("godex", PrintMode::Rfid),
            "3034257BF7194E406994036B",
        )
        .unwrap_err();

        assert!(matches!(err, PrintPipelineError::Core(_)));
        assert_eq!(err.to_string(), "godex does not support rfid");
    }

    #[test]
    fn rejects_missing_required_fields_from_core_plan() {
        let err = prepare_print_command(&reading(2.5), selection("zebra", PrintMode::Rfid), " ")
            .unwrap_err();

        assert!(matches!(err, PrintPipelineError::Core(_)));
        assert_eq!(
            err.to_string(),
            "zebra print job missing required fields: epc"
        );
    }
}
