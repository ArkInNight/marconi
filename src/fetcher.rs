//! Direct summon-history fetch.
//!
//! The game itself requests the summon history with a plain HTTPS GET to
//! `.../query/summon?userId=...&token=...` and that is exactly what the
//! community trackers replay. Doing the request here, in the same process
//! and second that the URL is captured, removes the clipboard round-trip
//! that used to lose the race against the token's ~290 s lifetime
//! (`SummonPoolHistoryModel:setToken` sets the expiry in the game client).
//!
//! Only used when the user explicitly passes `--json` / `--json-file`:
//! by default marconi still only prints/copies the link and sends nothing.

use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SummonHistoryResponse {
    pub code: i64,
    #[serde(default)]
    pub msg: String,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

impl SummonHistoryResponse {
    pub fn is_success(&self) -> bool {
        self.code == 200
    }

    /// Number of individual summon records in `data.pageData`, when present.
    pub fn record_count(&self) -> Option<usize> {
        self.data
            .as_ref()
            .and_then(|d| d.get("pageData"))
            .and_then(|p| p.as_array())
            .map(Vec::len)
    }
}

pub fn parse_response(body: &str) -> anyhow::Result<SummonHistoryResponse> {
    if body.trim().is_empty() {
        anyhow::bail!(
            "empty response from summon history server (the token is probably \
             invalid — the server only answers JSON for UUID-shaped tokens)"
        );
    }
    let parsed: SummonHistoryResponse = serde_json::from_str(body)
        .map_err(|e| anyhow::anyhow!("invalid JSON from summon history server: {e}"))?;
    Ok(parsed)
}

/// GET the summon history URL and return the raw response body.
///
/// The caller must have just captured `url`: the token inside it expires
/// in about five minutes, and the server answers an expired token with
/// HTTP 200 but body `{"msg":"没有权限","code":-6}`.
pub fn fetch_summon_history(url: &str, timeout: Duration) -> anyhow::Result<String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build();
    let mut response = ureq::Agent::new_with_config(config)
        .get(url)
        .header("Accept", "application/json")
        .call()
        .map_err(|e| anyhow::anyhow!("summon history request failed: {e}"))?;
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("could not read summon history response: {e}"))?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_response() {
        let body = r#"{"msg":"成功","code":200,"data":{"pageData":[{"gainIds":[1001],"poolType":3,"poolName":"x","createTime":123}]}}"#;
        let parsed = parse_response(body).unwrap();
        assert!(parsed.is_success());
        assert_eq!(parsed.record_count(), Some(1));
    }

    #[test]
    fn parses_expired_token_response() {
        let parsed = parse_response(r#"{"msg":"没有权限","code":-6}"#).unwrap();
        assert!(!parsed.is_success());
        assert_eq!(parsed.code, -6);
        assert_eq!(parsed.record_count(), None);
    }

    #[test]
    fn rejects_non_json() {
        assert!(parse_response("not json").is_err());
        assert!(parse_response("").is_err());
        assert!(parse_response("   ").is_err());
    }
}
