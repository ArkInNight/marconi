pub const SUMMON_ANCHOR: &str = "com/query/summon";

pub fn anchor_bytes_utf8() -> Vec<u8> {
    SUMMON_ANCHOR.as_bytes().to_vec()
}

pub fn anchor_bytes_utf16le() -> Vec<u8> {
    SUMMON_ANCHOR
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect()
}

pub struct ScanMatch {
    pub offset: usize,
    pub utf16: bool,
}

pub fn find_anchor(buf: &[u8], from: usize) -> Option<ScanMatch> {
    let utf8 = anchor_bytes_utf8();
    let utf16 = anchor_bytes_utf16le();

    let utf8_at = find_needle(buf, from, &utf8);
    let utf16_at = find_needle(buf, from, &utf16);

    match (utf8_at, utf16_at) {
        (Some(a), Some(b)) if a <= b => Some(ScanMatch {
            offset: a,
            utf16: false,
        }),
        (Some(_), Some(b)) => Some(ScanMatch {
            offset: b,
            utf16: true,
        }),
        (Some(a), None) => Some(ScanMatch {
            offset: a,
            utf16: false,
        }),
        (None, Some(b)) => Some(ScanMatch {
            offset: b,
            utf16: true,
        }),
        (None, None) => None,
    }
}

pub fn find_needle(buf: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > buf.len() {
        return None;
    }
    let max = buf.len() - needle.len();
    if from >= buf.len() || from > max {
        return None;
    }
    buf[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|pos| pos + from)
}
