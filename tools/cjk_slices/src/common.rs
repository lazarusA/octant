//! Each language's common characters, from its national standard's basic
//! set: GB 2312 level 1 for Chinese (3,755 hanzi, with its symbol rows),
//! JIS X 0208 level 1 for Japanese (2,965 kanji, with kana and symbols), and
//! KS X 1001 for Korean (its 2,350 Hangul syllables, jamo and symbols). A
//! character belongs when its legacy encoding (`encoding_rs`) is a two-byte
//! code in those rows with a trail byte of the base set (`0xA1..=0xFE`):
//! GBK and Windows-949 extend GB 2312 and KS X 1001 with lower trail bytes.

use encoding_rs::{EUC_JP, EUC_KR, Encoding, GBK};

/// Whether `c` is a common character of the font `code` (`sc`, `jp`, `kr`).
pub fn is_common(code: &str, c: char) -> bool {
    let (encoding, rows): (&Encoding, &[(u8, u8)]) = match code {
        "sc" => (GBK, &[(0xA1, 0xA9), (0xB0, 0xD7)]),
        "jp" => (EUC_JP, &[(0xA1, 0xA8), (0xB0, 0xCF)]),
        "kr" => (EUC_KR, &[(0xA1, 0xAC), (0xB0, 0xC8)]),
        _ => return false,
    };
    let mut buf = [0; 4];
    let (bytes, _, unmappable) = encoding.encode(c.encode_utf8(&mut buf));
    let [lead, trail] = bytes[..] else {
        return false;
    };
    !unmappable
        && (0xA1..=0xFE).contains(&trail)
        && rows.iter().any(|&(lo, hi)| (lo..=hi).contains(&lead))
}

#[cfg(test)]
mod tests {
    use super::is_common;

    #[test]
    fn common_sets_follow_the_national_standards() {
        // 中 (GB 2312 level 1), 龘 (rare, outside it).
        assert!(is_common("sc", '\u{4E2D}'));
        assert!(!is_common("sc", '\u{9F98}'));
        // 丟 (GBK extension, not GB 2312).
        assert!(!is_common("sc", '\u{4E1F}'));
        // あ (hiragana), 漢 (JIS level 1), 丂 (rare).
        assert!(is_common("jp", '\u{3042}'));
        assert!(is_common("jp", '\u{6F22}'));
        assert!(!is_common("jp", '\u{4E02}'));
        // 한 (KS X 1001), 똠 (outside its 2,350 syllables).
        assert!(is_common("kr", '\u{D55C}'));
        assert!(!is_common("kr", '\u{B620}'));
        assert!(!is_common("sc", 'a'));
    }
}
