pub fn copy(url: &str) -> anyhow::Result<()> {
    let mut clip = arboard::Clipboard::new()?;
    clip.set_text(url.to_string())?;
    Ok(())
}
