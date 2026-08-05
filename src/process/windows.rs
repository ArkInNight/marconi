use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub fn find_game_process_impl(pattern: Option<&str>) -> Option<(u32, String)> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return None;
    }

    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut best: Option<(u32, String)> = None;
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while ok {
        let name = wide_to_string(&entry.szExeFile);
        if matches_process(&name, pattern) {
            let pid = entry.th32ProcessID;
            if best.as_ref().map(|(p, _)| pid < *p).unwrap_or(true) {
                best = Some((pid, name));
            }
        }
        ok = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }

    unsafe { CloseHandle(snapshot) };
    best
}

fn wide_to_string(bytes: &[u16]) -> String {
    let len = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    String::from_utf16_lossy(&bytes[..len])
}

fn matches_process(name: &str, pattern: Option<&str>) -> bool {
    match pattern {
        Some(p) => name
            .to_ascii_lowercase()
            .contains(&p.to_ascii_lowercase()),
        None => name.to_ascii_lowercase().contains("reverse1999"),
    }
}
