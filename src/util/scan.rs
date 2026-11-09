pub const SUMMON_ANCHOR: &str = "com/query/summon";

static ANCHOR_UTF8: std::sync::LazyLock<Vec<u8>> =
    std::sync::LazyLock::new(|| SUMMON_ANCHOR.as_bytes().to_vec());
static ANCHOR_UTF16LE: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
    SUMMON_ANCHOR
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect()
});

pub struct ScanMatch {
    pub offset: usize,
    pub utf16: bool,
}

pub fn find_anchor(buf: &[u8], from: usize) -> Option<ScanMatch> {
    let utf8_at = find_needle(buf, from, &ANCHOR_UTF8);
    let utf16_at = find_needle(buf, from, &ANCHOR_UTF16LE);

    match (utf8_at, utf16_at) {
        (Some(a), Some(b)) if a <= b => Some(ScanMatch {
            offset: a,
            utf16: false,
        }),
        (Some(a), None) => Some(ScanMatch {
            offset: a,
            utf16: false,
        }),
        (_, Some(b)) => Some(ScanMatch {
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
    memchr::memmem::find(&buf[from..], needle).map(|pos| pos + from)
}
