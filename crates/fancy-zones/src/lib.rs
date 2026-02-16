pub mod layout;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct FancyZones {
    running: bool,
    layouts: Vec<layout::Layout>,
}

impl FancyZones {
    pub fn new() -> Self {
        Self {
            running: false,
            layouts: vec![layout::Layout::default_columns(3)],
        }
    }
}

impl Default for FancyZones {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for FancyZones {
    fn id(&self) -> &'static str {
        "fancy-zones"
    }

    fn name(&self) -> &'static str {
        "FancyZones"
    }

    fn description(&self) -> &'static str {
        "Advanced window tiling with custom zone layouts"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "Z"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("FancyZones started with {} layout(s)", self.layouts.len());
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("FancyZones stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("FancyZones: would open zone editor overlay");
        Ok(())
    }
}
