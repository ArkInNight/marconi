pub fn process_alive(_pid: u32) -> bool {
    // TODO(windows): check process liveness via OpenProcess. Phase 2.
    false
}

pub fn scan_process_memory(_pid: u32) -> Result<Option<String>, crate::error::AppError> {
    // TODO(windows): ReadProcessMemory / MiniDumpWriteDump + scan. Phase 2.
    Ok(None)
}
