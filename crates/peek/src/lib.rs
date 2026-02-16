pub mod preview;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct Peek {
    running: bool,
}

impl Peek {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for Peek {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for Peek {
    fn id(&self) -> &'static str {
        "peek"
    }

    fn name(&self) -> &'static str {
        "Peek"
    }

    fn description(&self) -> &'static str {
        "Quick file preview without opening the full application"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Ctrl], "Space"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Peek started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Peek stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Peek: would open file preview for selected file");
        Ok(())
    }
}
