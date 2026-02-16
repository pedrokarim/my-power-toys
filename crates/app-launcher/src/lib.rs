pub mod calculator;
pub mod desktop;
pub mod frecency;
pub mod search;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::info;

pub struct AppLauncher {
    running: bool,
    index: search::SearchIndex,
}

impl AppLauncher {
    pub fn new() -> Self {
        Self {
            running: false,
            index: search::SearchIndex::new(),
        }
    }
}

impl Default for AppLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for AppLauncher {
    fn id(&self) -> &'static str {
        "app-launcher"
    }

    fn name(&self) -> &'static str {
        "App Launcher"
    }

    fn description(&self) -> &'static str {
        "Quick application launcher with search, frecency ranking and inline calculator"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Alt], "Space"))
    }

    fn start(&mut self) -> Result<()> {
        info!("App Launcher: indexing .desktop files...");
        self.index.refresh()?;
        info!(
            "App Launcher: indexed {} applications",
            self.index.app_count()
        );
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("App Launcher stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("App Launcher: would open launcher overlay");
        Ok(())
    }
}
