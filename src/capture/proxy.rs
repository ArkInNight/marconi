use super::{CaptureConfig, CaptureEngine};
use crate::output::Output;
use std::pin::Pin;

pub struct ProxyEngine;

impl CaptureEngine for ProxyEngine {
    fn name(&self) -> &'static str {
        "proxy"
    }

    fn capture<'a>(
        &'a mut self,
        _config: &'a CaptureConfig,
        _output: &'a Output,
    ) -> Pin<Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send + 'a>>
    {
        // Strategy 2: temporary HTTPS proxy. Not implemented in this iteration.
        Box::pin(async move { Ok(None) })
    }
}
