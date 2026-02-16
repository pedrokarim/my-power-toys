mod clipboard;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct PastePlain {
    running: bool,
}

impl PastePlain {
    pub fn new() -> Self {
        Self { running: false }
    }

    /// Read clipboard, strip formatting, write back as plain text, then simulate Ctrl+V.
    pub fn paste_as_plain(&self) -> Result<()> {
        let text = clipboard::get_text()?;
        if text.is_empty() {
            info!("Clipboard is empty, nothing to paste");
            return Ok(());
        }
        clipboard::set_text(&text)?;
        clipboard::simulate_paste()?;
        info!("Pasted {} chars as plain text", text.len());
        Ok(())
    }
}

impl Default for PastePlain {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for PastePlain {
    fn id(&self) -> &'static str {
        "paste-plain"
    }

    fn name(&self) -> &'static str {
        "Paste as Plain Text"
    }

    fn description(&self) -> &'static str {
        "Paste clipboard contents as plain text, stripping all formatting"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Ctrl], "V"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Paste as Plain Text module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Paste as Plain Text module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        self.paste_as_plain()
    }
}
