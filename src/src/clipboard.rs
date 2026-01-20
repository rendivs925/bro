use anyhow::{Context, Result};

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("Failed to access system clipboard")?;
    clipboard
        .set_text(text.to_string())
        .context("Failed to set clipboard text")?;
    Ok(())
}
