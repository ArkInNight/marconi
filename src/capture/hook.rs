use super::{CaptureConfig, CaptureEngine};
use crate::output::Output;
use std::pin::Pin;

pub struct HookEngine;

impl CaptureEngine for HookEngine {
    fn name(&self) -> &'static str {
        "hook"
    }

    fn capture<'a>(
        &'a mut self,
        _config: &'a CaptureConfig,
        _output: &'a Output,
    ) -> Pin<Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send + 'a>>
    {
        // Strategy 1b: native hook (WinHTTP/libcurl/OpenSSL). Not implemented.
        Box::pin(async move { Ok(None) })
    }
}
