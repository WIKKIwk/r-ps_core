use crate::print::mode::PrintMode;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuantitySource {
    Scale,
    Manual,
}

impl QuantitySource {
    pub fn normalize(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "manual" | "manual_kg" | "manual-kg" | "kg" => Self::Manual,
            _ => Self::Scale,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scale => "scale",
            Self::Manual => "manual",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintSelection {
    pub item_code: String,
    pub item_name: String,
    pub warehouse: String,
    pub print_mode: PrintMode,
    pub printer: String,
    pub quantity_source: QuantitySource,
    pub manual_qty_kg: f64,
    pub tare_enabled: bool,
    pub tare_kg: f64,
}

impl PrintSelection {
    pub fn normalized(mut self) -> Self {
        self.item_code = self.item_code.trim().to_string();
        self.item_name = self.item_name.trim().to_string();
        self.warehouse = self.warehouse.trim().to_string();
        self.printer = self.printer.trim().to_ascii_lowercase();
        if self.quantity_source != QuantitySource::Manual || self.manual_qty_kg <= 0.0 {
            self.manual_qty_kg = 0.0;
        }
        if !self.tare_enabled || self.tare_kg <= 0.0 {
            self.tare_enabled = false;
            self.tare_kg = 0.0;
        }
        if self.item_name.is_empty() {
            self.item_name = self.item_code.clone();
        }
        self
    }

    pub fn uses_manual_qty(&self) -> bool {
        self.quantity_source == QuantitySource::Manual
    }

    pub fn net_qty(&self, gross_qty: f64) -> f64 {
        if !self.tare_enabled {
            return gross_qty;
        }
        (gross_qty - self.tare_kg).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection() -> PrintSelection {
        PrintSelection {
            item_code: " ITEM-1 ".to_string(),
            item_name: " ".to_string(),
            warehouse: " Stores - A ".to_string(),
            print_mode: PrintMode::Rfid,
            printer: " GoDEX ".to_string(),
            quantity_source: QuantitySource::Scale,
            manual_qty_kg: 7.0,
            tare_enabled: true,
            tare_kg: 0.5,
        }
    }

    #[test]
    fn normalizes_selection_like_gscale_workflow() {
        let normalized = selection().normalized();

        assert_eq!(normalized.item_code, "ITEM-1");
        assert_eq!(normalized.item_name, "ITEM-1");
        assert_eq!(normalized.warehouse, "Stores - A");
        assert_eq!(normalized.printer, "godex");
        assert_eq!(normalized.manual_qty_kg, 0.0);
        assert!(normalized.tare_enabled);
        assert_eq!(normalized.tare_kg, 0.5);
    }

    #[test]
    fn normalizes_quantity_source_like_gscale_workflow() {
        assert_eq!(
            QuantitySource::normalize("manual_kg"),
            QuantitySource::Manual
        );
        assert_eq!(
            QuantitySource::normalize("manual-kg"),
            QuantitySource::Manual
        );
        assert_eq!(QuantitySource::normalize("kg"), QuantitySource::Manual);
        assert_eq!(QuantitySource::normalize("scale"), QuantitySource::Scale);
        assert_eq!(QuantitySource::normalize("unknown"), QuantitySource::Scale);
    }

    #[test]
    fn computes_net_qty_like_gscale_workflow() {
        let normalized = selection().normalized();

        assert_eq!(normalized.net_qty(2.5), 2.0);
        assert_eq!(normalized.net_qty(0.3), 0.0);
    }

    #[test]
    fn disables_invalid_tare_and_manual_qty_like_gscale_workflow() {
        let mut input = selection();
        input.quantity_source = QuantitySource::Manual;
        input.manual_qty_kg = -1.0;
        input.tare_kg = 0.0;

        let normalized = input.normalized();

        assert_eq!(normalized.manual_qty_kg, 0.0);
        assert!(!normalized.tare_enabled);
        assert_eq!(normalized.tare_kg, 0.0);
    }
}
