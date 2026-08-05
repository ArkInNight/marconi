pub fn find_game_process_impl(_pattern: Option<&str>) -> Option<(u32, String)> {
    // TODO(windows): enumerate processes via Toolhelp32Snapshot / EnumProcesses
    // and match Reverse1999.exe. Phase 2. For now return None.
    None
}
