#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

use super::{CaptureConfig, CaptureEngine};
use crate::output::Output;
use crate::url_extractor::{extract_url, is_valid_url};
use crate::util::scan::{find_anchor, SUMMON_ANCHOR};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
use linux::{process_alive, scan_process_memory};

#[cfg(target_os = "windows")]
use windows::{process_alive, scan_process_memory};

const CHUNK_SIZE: usize = 1024 * 1024;
const OVERLAP: usize = 8192;
const POLL_INTERVAL: Duration = Duration::from_secs(4);
const FILE_MAPPED_AFTER_PASSES: u32 = 3;

/// Scans a stream of chunks while keeping `OVERLAP` bytes of the previous
/// chunk so strings spanning chunk boundaries are still matched. Buffers are
/// reused across chunks to avoid repeated allocations.
struct ChunkedScanner {
    carry: Vec<u8>,
    combined: Vec<u8>,
}

impl ChunkedScanner {
    fn new() -> Self {
        Self {
            carry: Vec::new(),
            combined: Vec::with_capacity(CHUNK_SIZE + OVERLAP),
        }
    }

    fn push_and_scan(&mut self, buf: &[u8]) -> Option<String> {
        self.combined.clear();
        self.combined.extend_from_slice(&self.carry);
        self.combined.extend_from_slice(buf);
        let result = MemScanEngine::scan_buffer(&self.combined);
        let keep = OVERLAP.min(self.combined.len());
        self.carry.clear();
        self.carry
            .extend_from_slice(&self.combined[self.combined.len() - keep..]);
        result
    }
}

#[derive(Default)]
pub struct MemScanEngine {
    dump_file: Option<PathBuf>,
}

impl MemScanEngine {
    pub fn new(dump_file: Option<PathBuf>) -> Self {
        Self { dump_file }
    }

    pub fn scan_buffer(buf: &[u8]) -> Option<String> {
        let mut from = 0;
        while let Some(m) = find_anchor(buf, from) {
            if let Some(url) = extract_url(buf, &m) {
                if is_valid_url(&url) {
                    return Some(url);
                }
            }
            from = m.offset + SUMMON_ANCHOR.len();
        }
        None
    }
}

fn scan_dump_file(path: &Path) -> anyhow::Result<Option<String>> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut scanner = ChunkedScanner::new();
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if let Some(url) = scanner.push_and_scan(&buf[..n]) {
            return Ok(Some(url));
        }
        if n < CHUNK_SIZE {
            break;
        }
    }
    Ok(None)
}

impl CaptureEngine for MemScanEngine {
    fn name(&self) -> &'static str {
        "memory-scan"
    }

    fn capture<'a>(
        &'a mut self,
        config: &'a CaptureConfig,
        output: &'a Output,
    ) -> Pin<Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send + 'a>>
    {
        Box::pin(async move {
            if let Some(path) = &self.dump_file {
                let path = path.clone();
                let inner = tokio::task::spawn_blocking(move || scan_dump_file(&path)).await?;
                return inner;
            }

            let Some(pid) = config.game_pid else {
                return Ok(None);
            };

            let deadline = Instant::now() + config.timeout;
            let mut denied = false;
            let mut passes = 0;
            loop {
                if !process_alive(pid) {
                    if output.verbose {
                        println!("[capture] game process {pid} exited");
                    }
                    return Ok(None);
                }
                output.scanning();
                let include_file_mapped = passes >= FILE_MAPPED_AFTER_PASSES;
                let verbose = output.verbose;
                let result = tokio::task::spawn_blocking(move || {
                    scan_process_memory(pid, verbose, include_file_mapped)
                })
                .await
                .map_err(|join| anyhow::anyhow!("memory scan task panicked: {join}"))?;
                match result {
                    Ok(Some(url)) => return Ok(Some(url)),
                    Ok(None) => {}
                    Err(err) => {
                        if matches!(err, crate::error::AppError::AccessDenied) {
                            if !denied {
                                denied = true;
                                println!("! Cannot read game memory (access denied).");
                                #[cfg(target_os = "linux")]
                                {
                                    println!("  Fix: run this tool with sudo, or temporarily");
                                    println!("  run: sudo sysctl -w kernel.yama.ptrace_scope=0");
                                }
                                println!();
                            }
                        } else if output.verbose {
                            println!("[capture] scan error: {err}");
                        }
                    }
                }
                if Instant::now() >= deadline {
                    return Ok(None);
                }
                passes += 1;
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_url() -> &'static str {
        "https://game-re-en-service.sl916.com/query/summon?userId=u&token=t&sign=s"
    }

    fn pad_to_chunk(s: &str, target: usize) -> Vec<u8> {
        let mut v = vec![0u8; target];
        v[..s.len()].copy_from_slice(s.as_bytes());
        v
    }

    #[test]
    fn finds_url_in_utf8_dump() {
        let mut dump = Vec::new();
        for _ in 0..3 {
            dump.extend_from_slice(&pad_to_chunk(sample_url(), CHUNK_SIZE));
        }
        assert_eq!(
            MemScanEngine::scan_buffer(&dump).as_deref(),
            Some(sample_url())
        );
    }

    #[test]
    fn finds_url_in_utf16_dump() {
        let utf16: Vec<u8> = sample_url()
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut dump = vec![0xAB; 1000];
        dump.extend_from_slice(&utf16);
        dump.extend(vec![0u8; 64]);
        assert_eq!(
            MemScanEngine::scan_buffer(&dump).as_deref(),
            Some(sample_url())
        );
    }

    #[test]
    fn crosses_chunk_boundary() {
        let url = sample_url();
        let mut buf = vec![0u8; CHUNK_SIZE - 40];
        buf.extend_from_slice(url.as_bytes());
        assert_eq!(MemScanEngine::scan_buffer(&buf).as_deref(), Some(url));
    }

    #[test]
    fn scan_dump_file_works() {
        let dir = std::env::temp_dir().join("marconi-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("sample.dmp");
        let mut bytes = "noise".repeat(20).into_bytes();
        bytes.extend_from_slice(sample_url().as_bytes());
        bytes.extend(vec![0u8; 5000]);
        std::fs::write(&path, &bytes).unwrap();

        let result = scan_dump_file(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(result.as_deref(), Some(sample_url()));
    }

    #[test]
    fn empty_dump_returns_none() {
        let dir = std::env::temp_dir().join("marconi-test-empty");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.dmp");
        std::fs::write(&path, b"hello world").unwrap();
        let result = scan_dump_file(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_none());
    }
}
