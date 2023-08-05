#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZebraWeightBlock {
    pub zpl: String,
    pub epc_y: i32,
    pub barcode_y: i32,
}

pub fn build_zebra_weight_block(netto_text: &str, brutto_text: &str) -> ZebraWeightBlock {
    if brutto_text.trim().is_empty() {
        return ZebraWeightBlock {
            zpl: "^FO8,118^A0N,44,38\n".to_string() + &format!("^FDVAZNI: {netto_text}^FS\n"),
            epc_y: 184,
            barcode_y: 236,
        };
    }

    ZebraWeightBlock {
        zpl: "^FO8,112^A0N,36,32\n".to_string()
            + &format!("^FDNETTO: {netto_text}^FS\n")
            + "^FO8,158^A0N,36,32\n"
            + &format!("^FDBRUTTO: {brutto_text}^FS\n"),
        epc_y: 220,
        barcode_y: 272,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_single_weight_line_without_tare() {
        let block = build_zebra_weight_block("1.2 kg", "");

        assert!(block.zpl.contains("^FDVAZNI: 1.2 kg^FS"));
        assert_eq!(block.epc_y, 184);
        assert_eq!(block.barcode_y, 236);
    }

    #[test]
    fn builds_netto_and_brutto_lines_with_tare() {
        let block = build_zebra_weight_block("1.7 kg", "2.5 kg");

        assert!(block.zpl.contains("^FDNETTO: 1.7 kg^FS"));
        assert!(block.zpl.contains("^FDBRUTTO: 2.5 kg^FS"));
        assert_eq!(block.epc_y, 220);
        assert_eq!(block.barcode_y, 272);
    }
}
