use crate::util::scan::{ScanMatch, SUMMON_ANCHOR};

const MAX_PREFIX_BACKSCAN: usize = 2048;
const MAX_URL_TAIL: usize = 4096;

fn is_url_ascii(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"-._~:/?#[]@!$&'()*+,;=%".contains(&b)
}

fn find_prefix_backward(buf: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    let min = from.saturating_sub(MAX_PREFIX_BACKSCAN);
    let mut pos = from;
    while pos > min {
        let start = pos - 1;
        if start + needle.len() <= buf.len() && buf[start..start + needle.len()] == *needle {
            return Some(start);
        }
        pos -= 1;
    }
    None
}

fn collect_url_tail(buf: &[u8], from: usize, utf16: bool) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = from;
    if utf16 {
        while i + 1 < buf.len() {
            let (lo, hi) = (buf[i], buf[i + 1]);
            if hi != 0 || !is_url_ascii(lo) || out.len() >= MAX_URL_TAIL {
                break;
            }
            out.push(lo);
            i += 2;
        }
    } else {
        while i < buf.len() {
            let b = buf[i];
            if !is_url_ascii(b) || out.len() >= MAX_URL_TAIL {
                break;
            }
            out.push(b);
            i += 1;
        }
    }
    out
}

fn decode_run(buf: &[u8], start: usize, end: usize, utf16: bool) -> String {
    let bytes = &buf[start..end];
    if utf16 {
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn trim_url_garbage(mut url: String) -> String {
    let trim: &[char] = &[')', ',', ';', '.', '\'', '"'];
    while url.len() > 1 {
        let last = url.chars().last().unwrap();
        if !trim.contains(&last) {
            break;
        }
        url.pop();
    }
    url
}

pub fn extract_url(buf: &[u8], m: &ScanMatch) -> Option<String> {
    let anchor_len = SUMMON_ANCHOR.len();
    let anchor_end = if m.utf16 {
        m.offset + anchor_len * 2
    } else {
        m.offset + anchor_len
    };
    if m.offset >= buf.len() || anchor_end > buf.len() {
        return None;
    }

    let (prefix_needle, scheme) = if m.utf16 {
        let n: Vec<u8> = "https://"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        let http: Vec<u8> = "http://"
            .encode_utf16()
            .flat_map(|u| u.to_le_bytes())
            .collect();
        (vec![n, http], "https://")
    } else {
        (vec![b"https://".to_vec(), b"http://".to_vec()], "https://")
    };

    let prefix_start = prefix_needle
        .iter()
        .filter_map(|n| find_prefix_backward(buf, m.offset, n))
        .min()
        .unwrap_or(0);

    let tail = collect_url_tail(buf, anchor_end, m.utf16);
    if tail.is_empty() {
        return None;
    }
    let tail_bytes = if m.utf16 { tail.len() * 2 } else { tail.len() };
    let tail_end = (anchor_end + tail_bytes).min(buf.len());

    let mut full = decode_run(buf, prefix_start, tail_end, m.utf16);
    if !full.starts_with("https://") && !full.starts_with("http://") {
        full = format!("{scheme}{full}");
    }
    full = full.replace("&amp;", "&");
    Some(trim_url_garbage(full))
}

pub fn is_valid_url(url: &str) -> bool {
    if !url.contains("/query/summon") {
        return false;
    }
    match url.split_once('?') {
        Some((_, query)) => !query.is_empty(),
        None => false,
    }
}

pub fn sanitize_url(url: &str) -> String {
    let (base, query) = match url.split_once('?') {
        Some(v) => v,
        None => return url.to_string(),
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|p| {
            let key = p.split('=').next().unwrap_or("");
            matches!(key, "userId" | "token")
        })
        .collect();
    if kept.is_empty() {
        return url.to_string();
    }
    format!("{base}?{}", kept.join("&"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf8_buf(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    fn utf16_buf(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect()
    }

    fn match_at(buf: &[u8], utf16: bool) -> ScanMatch {
        let needle = if utf16 {
            SUMMON_ANCHOR
                .encode_utf16()
                .flat_map(|u| u.to_le_bytes())
                .collect::<Vec<_>>()
        } else {
            SUMMON_ANCHOR.as_bytes().to_vec()
        };
        let off = crate::util::scan::find_needle(buf, 0, &needle).unwrap();
        ScanMatch { offset: off, utf16 }
    }

    #[test]
    fn extract_utf8_url() {
        let url = "https://game-re-en-service.sl916.com/query/summon?a=1&b=2";
        let buf = utf8_buf(url);
        let m = match_at(&buf, false);
        assert_eq!(extract_url(&buf, &m).unwrap(), url);
    }

    #[test]
    fn extract_utf16_url() {
        let url = "https://game-re-en-service.sl916.com/query/summon?a=1&b=2";
        let buf = utf16_buf(url);
        let m = match_at(&buf, true);
        assert_eq!(extract_url(&buf, &m).unwrap(), url);
    }

    #[test]
    fn extract_from_noisy_window() {
        let noisy = "x".repeat(300)
            + "https://game-re-en-service.sl916.com/query/summon?userId=u&token=t&sign=s"
            + "\0"
            + &"y".repeat(200);
        let buf = utf8_buf(&noisy);
        let m = match_at(&buf, false);
        assert_eq!(
            extract_url(&buf, &m).unwrap(),
            "https://game-re-en-service.sl916.com/query/summon?userId=u&token=t&sign=s"
        );
    }

    #[test]
    fn unescapes_amp() {
        let url = "https://game-re-en-service.sl916.com/query/summon?a=1&amp;b=2";
        let buf = utf8_buf(url);
        let m = match_at(&buf, false);
        assert_eq!(
            extract_url(&buf, &m).unwrap(),
            "https://game-re-en-service.sl916.com/query/summon?a=1&b=2"
        );
    }

    #[test]
    fn trims_trailing_garbage() {
        let url = "https://game-re-en-service.sl916.com/query/summon?a=1\x00\");";
        let buf = utf8_buf(url);
        let m = match_at(&buf, false);
        assert_eq!(
            extract_url(&buf, &m).unwrap(),
            "https://game-re-en-service.sl916.com/query/summon?a=1"
        );
    }

    #[test]
    fn validation_rules() {
        assert!(is_valid_url(
            "https://game-re-en-service.sl916.com/query/summon?a=1"
        ));
        assert!(!is_valid_url("https://example.com/query/summon"));
        assert!(!is_valid_url(
            "https://game-re-en-service.sl916.com/other?x=1"
        ));
        assert!(!is_valid_url(
            "https://game-re-en-service.sl916.com/query/summon"
        ));
    }

    #[test]
    fn sanitize_keeps_user_and_token() {
        let url = "https://game-re-en-service.sl916.com/query/summon?timestamp=1&sign=s&userId=u&token=t&version=1.4";
        assert_eq!(
            sanitize_url(url),
            "https://game-re-en-service.sl916.com/query/summon?userId=u&token=t"
        );
    }
}
