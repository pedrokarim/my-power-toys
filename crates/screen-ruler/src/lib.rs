pub mod measurement;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct ScreenRuler {
    running: bool,
}

impl ScreenRuler {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for ScreenRuler {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for ScreenRuler {
    fn id(&self) -> &'static str {
        "screen-ruler"
    }

    fn name(&self) -> &'static str {
        "Screen Ruler"
    }

    fn description(&self) -> &'static str {
        "Measure pixel distances on screen with an overlay"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "R"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Screen Ruler module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Screen Ruler module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Screen Ruler: would open measurement overlay");
        Ok(())
    }
}
