//! Allocation-free formatting into caller-provided stack buffers.

use std::io::Write;

/// Format `args` into `buf` and return the written text, truncating at a
/// UTF-8 boundary if it does not fit. Use with `format_args!` in per-frame
/// UI code instead of `format!`.
pub fn stack_str<'a>(buf: &'a mut [u8], args: std::fmt::Arguments<'_>) -> &'a str {
    let mut cursor = std::io::Cursor::new(&mut buf[..]);
    // A full buffer only truncates; whatever was written is still valid.
    let _ = cursor.write_fmt(args);
    let len = usize::try_from(cursor.position()).unwrap_or(0);
    match std::str::from_utf8(&buf[..len]) {
        Ok(s) => s,
        // Truncation may split a multi-byte char; keep the valid prefix.
        Err(e) => std::str::from_utf8(&buf[..e.valid_up_to()]).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::stack_str;
    use crate::utils::ByteSize;

    #[test]
    fn test_formats_into_buffer() {
        let mut buf = [0u8; 32];
        assert_eq!(
            stack_str(&mut buf, format_args!("{} / {}", 3, "x")),
            "3 / x"
        );
    }

    #[test]
    fn test_truncates_when_full() {
        let mut buf = [0u8; 4];
        assert_eq!(stack_str(&mut buf, format_args!("abcdef")), "abcd");
    }

    #[test]
    fn test_byte_size_matches_string_formatter() {
        for bytes in [
            0,
            512,
            1024,
            1536,
            50 * 1024 * 1024,
            3 * 1024_u64.pow(4) / 2,
        ] {
            let mut buf = [0u8; 32];
            let stacked = stack_str(&mut buf, format_args!("{}", ByteSize(bytes)));
            assert_eq!(stacked, crate::utils::format_byte_size(bytes));
        }
    }
}
