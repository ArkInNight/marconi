pub fn copy(url: &str) -> anyhow::Result<()> {
    let mut clip = arboard::Clipboard::new()?;
    clip.set_text(url.to_string())?;

    // Wayland serves clipboard data on demand from the owning client. If this
    // process exits right away, the compositor (and Klipper on Plasma) may not
    // have fetched the text yet, leaving the clipboard empty. Stay alive a
    // moment so the data actually lands before the connection closes.
    #[cfg(target_os = "linux")]
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}
