pub use crate::core::label::{DEFAULT_QR_BASE_URL, encode_scan_payload};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_scan_payload_like_gscale() {
        assert_eq!(
            encode_scan_payload(
                "Accord LLC",
                "Green Tea 1kg",
                "1.2",
                "5",
                "3034257BF7194E406994036B"
            ),
            "https://scan.wspace.sbs/L/Accord+LLC/Green+Tea+1kg/1.2/5/3034257BF7194E406994036B"
        );
    }

    #[test]
    fn query_escapes_utf8_and_reserved_chars_like_go() {
        assert_eq!(
            encode_scan_payload("A+B", "O'zbek чой", "1,2", "5 kg", "ABC/123"),
            "https://scan.wspace.sbs/L/A%2BB/O%27zbek+%D1%87%D0%BE%D0%B9/1%2C2/5+kg/ABC%2F123"
        );
    }
}
