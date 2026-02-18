pub mod calculator;
pub mod desktop;
pub mod frecency;
pub mod search;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLauncherConfig {
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    #[serde(default = "default_true")]
    pub show_calculator: bool,
}

fn default_max_results() -> usize {
    8
}

fn default_true() -> bool {
    true
}

impl Default for AppLauncherConfig {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            show_calculator: default_true(),
        }
    }
}

pub struct AppLauncher {
    running: bool,
    index: search::SearchIndex,
    config: AppLauncherConfig,
}

impl AppLauncher {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("app-launcher").unwrap_or_default();
        Self {
            running: false,
            index: search::SearchIndex::new(),
            config,
        }
    }

    pub fn config(&self) -> &AppLauncherConfig {
        &self.config
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = AppLauncherConfig::default();
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: AppLauncherConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.max_results, 8);
        assert!(parsed.show_calculator);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let parsed: AppLauncherConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.max_results, 8);
        assert!(parsed.show_calculator);
    }
}
