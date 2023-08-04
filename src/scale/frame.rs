pub fn pop_serial_frame(buffer: &str) -> Option<(String, String)> {
    let index = buffer.find(['\r', '\n'])?;
    let frame = buffer[..index].to_string();
    let mut rest_index = index;

    for (offset, ch) in buffer[index..].char_indices() {
        if ch != '\r' && ch != '\n' {
            rest_index = index + offset;
            break;
        }
        rest_index = index + offset + ch.len_utf8();
    }

    Some((frame, buffer[rest_index..].to_string()))
}

pub fn append_raw(existing: &str, chunk: &str, max: usize) -> String {
    let combined = format!("{existing}{chunk}");
    if combined.len() <= max {
        return combined;
    }
    combined[combined.len() - max..].to_string()
}

pub fn sanitize_inline(raw: &str, max: usize) -> String {
    let mut value = raw.replace('\r', "\\r").replace('\n', "\\n");
    value = value.trim().to_string();
    if value.len() <= max {
        return value;
    }
    value[value.len() - max..].to_string()
}

#[cfg(test)]
mod tests {
    use super::{append_raw, pop_serial_frame, sanitize_inline};

    #[test]
    fn pops_serial_frame_like_go() {
        let (frame, rest) = pop_serial_frame("-  2.05\r-  1.00\r").expect("frame");
        assert_eq!(frame, "-  2.05");
        assert_eq!(rest, "-  1.00\r");
    }

    #[test]
    fn consumes_contiguous_crlf_like_go() {
        let (frame, rest) = pop_serial_frame("0.00\r\n- 0.50\n").expect("frame");
        assert_eq!(frame, "0.00");
        assert_eq!(rest, "- 0.50\n");
    }

    #[test]
    fn returns_none_without_delimiter_like_go() {
        assert!(pop_serial_frame("-  2.05").is_none());
    }

    #[test]
    fn append_raw_keeps_suffix_like_go() {
        assert_eq!(append_raw("12345", "678", 5), "45678");
        assert_eq!(append_raw("12", "34", 5), "1234");
    }

    #[test]
    fn sanitize_inline_escapes_and_keeps_suffix_like_go() {
        assert_eq!(sanitize_inline("  a\rb\n  ", 20), "a\\rb\\n");
        assert_eq!(sanitize_inline("123456789", 4), "6789");
    }
}
