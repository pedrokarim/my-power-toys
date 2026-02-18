pub mod preview;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeekConfig {
    #[serde(default = "default_max_preview_lines")]
    pub max_preview_lines: usize,

    #[serde(default = "default_max_dir_entries")]
    pub max_dir_entries: usize,
}

fn default_max_preview_lines() -> usize {
    50
}

fn default_max_dir_entries() -> usize {
    20
}

impl Default for PeekConfig {
    fn default() -> Self {
        Self {
            max_preview_lines: default_max_preview_lines(),
            max_dir_entries: default_max_dir_entries(),
        }
    }
}

pub struct Peek {
    running: bool,
    config: PeekConfig,
}

impl Peek {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("peek").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &PeekConfig {
        &self.config
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = PeekConfig::default();
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: PeekConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.max_preview_lines, 50);
        assert_eq!(parsed.max_dir_entries, 20);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let parsed: PeekConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.max_preview_lines, 50);
        assert_eq!(parsed.max_dir_entries, 20);
    }
}
