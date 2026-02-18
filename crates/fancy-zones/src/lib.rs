pub mod config;
pub mod layout;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct FancyZones {
    running: bool,
    config: config::FancyZonesConfig,
}

impl FancyZones {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("fancy-zones").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &config::FancyZonesConfig {
        &self.config
    }

    /// Get the currently active layout.
    pub fn active_layout(&self) -> Option<&layout::Layout> {
        self.config.layouts.get(self.config.active_layout)
    }

    /// Save current config to disk.
    pub fn save_config(&self) -> Result<()> {
        mpt_common::config::save_module_config("fancy-zones", &self.config)
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
        info!(
            "FancyZones started with {} layout(s)",
            self.config.layouts.len()
        );
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
