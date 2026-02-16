pub mod shortcuts;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct ShortcutGuide {
    running: bool,
}

impl ShortcutGuide {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for ShortcutGuide {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for ShortcutGuide {
    fn id(&self) -> &'static str {
        "shortcut-guide"
    }

    fn name(&self) -> &'static str {
        "Shortcut Guide"
    }

    fn description(&self) -> &'static str {
        "Show an overlay with available keyboard shortcuts when holding Super"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super], "?"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Shortcut Guide module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Shortcut Guide module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Shortcut Guide: would show overlay");
        Ok(())
    }
}
