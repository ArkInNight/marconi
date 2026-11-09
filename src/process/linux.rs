use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use sysinfo::{ProcessesToUpdate, System};

pub fn find_game_process_impl(pattern: Option<&str>) -> Option<(u32, String)> {
    let excluded = ancestor_pids();

    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut best: Option<(u32, String)> = None;
    for (pid, process) in sys.processes() {
        if excluded.contains(&pid.as_u32()) {
            continue;
        }
        if !is_game_process(pid.as_u32(), process.name(), pattern) {
            continue;
        }
        let name = process.name().to_string_lossy().into_owned();
        if best.as_ref().is_none_or(|(p, _)| *p > pid.as_u32()) {
            best = Some((pid.as_u32(), name));
        }
    }
    best
}

/// PIDs of this process and all its ancestors; never report these as the game.
fn ancestor_pids() -> Vec<u32> {
    let mut pids = Vec::new();
    let mut current = std::process::id();
    loop {
        pids.push(current);
        let Ok(stat) = std::fs::read_to_string(format!("/proc/{current}/stat")) else {
            break;
        };
        let Some(after_comm) = stat.rsplit(')').next() else {
            break;
        };
        let fields: Vec<&str> = after_comm.split_whitespace().collect();
        let Some(ppid) = fields.get(1).and_then(|f| f.parse().ok()) else {
            break;
        };
        if ppid == current {
            break;
        }
        current = ppid;
    }
    pids
}

fn matches_any(haystacks: &[String], pattern: &str) -> bool {
    let lower = pattern.to_ascii_lowercase();
    haystacks
        .iter()
        .any(|h| h.to_ascii_lowercase().contains(&lower))
}

fn is_game_process(pid: u32, name: &OsStr, pattern: Option<&str>) -> bool {
    let proc_name = name.to_string_lossy().into_owned();

    let cmdline_path = format!("/proc/{pid}/cmdline");
    let cmd0 = std::fs::read(&cmdline_path).ok().and_then(|raw| {
        raw.split(|b| *b == 0)
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
    });

    let exe_link = std::fs::read_link(format!("/proc/{pid}/exe"))
        .ok()
        .map(|t| String::from_utf8_lossy(t.as_os_str().as_bytes()).into_owned());

    if let Some(p) = pattern {
        let hay: Vec<String> = [Some(proc_name), cmd0, exe_link]
            .into_iter()
            .flatten()
            .collect();
        matches_any(&hay, p)
    } else {
        let name_lower = proc_name.to_ascii_lowercase();
        if name_lower.contains("reverse1999") {
            return true;
        }
        if let Some(c0) = &cmd0 {
            let c0_lower = c0.to_ascii_lowercase();
            if c0_lower.contains("reverse1999") || c0_lower.contains("3092660") {
                return true;
            }
        }
        if let Some(exe) = &exe_link {
            if exe.to_ascii_lowercase().contains("reverse1999") {
                return true;
            }
        }
        false
    }
}
