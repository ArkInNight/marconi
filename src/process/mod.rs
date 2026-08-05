#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
use linux::find_game_process_impl;

#[cfg(target_os = "windows")]
use windows::find_game_process_impl;

pub async fn wait_for_game(
    pattern: Option<&str>,
    timeout: Option<std::time::Duration>,
) -> Option<(u32, String)> {
    let start = std::time::Instant::now();
    loop {
        if let Some(found) = find_game_process_impl(pattern) {
            return Some(found);
        }
        if let Some(t) = timeout {
            if start.elapsed() >= t {
                return None;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    }
}
