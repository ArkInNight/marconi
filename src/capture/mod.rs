pub mod hook;
pub mod memscan;
pub mod proxy;

use crate::output::Output;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub struct CaptureConfig {
    pub game_pid: Option<u32>,
    pub timeout: Duration,
}

pub trait CaptureEngine {
    fn name(&self) -> &'static str;
    fn capture<'a>(
        &'a mut self,
        config: &'a CaptureConfig,
        output: &'a Output,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Option<String>>> + Send + 'a>>;
}

pub async fn run_strategies(
    config: &CaptureConfig,
    output: &Output,
) -> anyhow::Result<Option<String>> {
    let mut engines: Vec<Box<dyn CaptureEngine>> = vec![
        Box::new(memscan::MemScanEngine::default()),
        Box::new(proxy::ProxyEngine),
        Box::new(hook::HookEngine),
    ];

    for engine in engines.iter_mut() {
        if output.verbose {
            println!("[capture] trying strategy: {}", engine.name());
        }
        if let Some(url) = engine.capture(config, output).await? {
            return Ok(Some(url));
        }
    }
    Ok(None)
}
