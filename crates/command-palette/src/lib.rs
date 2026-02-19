pub mod frecency;
#[cfg(feature = "gui")]
pub mod gui;
pub mod providers;
pub mod search;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

// ── Configuration ───────────────────────────────────────────────────────────

fn default_max_results() -> usize {
    8
}

fn default_search_engine() -> String {
    "google".into()
}

fn default_file_tool() -> String {
    "auto".into()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    #[serde(default = "default_true")]
    pub apps: bool,
    #[serde(default = "default_true")]
    pub calculator: bool,
    #[serde(default = "default_true")]
    pub web_search: bool,
    #[serde(default = "default_true")]
    pub shell_commands: bool,
    #[serde(default = "default_true")]
    pub system_commands: bool,
    #[serde(default = "default_true")]
    pub file_search: bool,
    #[serde(default = "default_true")]
    pub settings: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            apps: true,
            calculator: true,
            web_search: true,
            shell_commands: true,
            system_commands: true,
            file_search: true,
            settings: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPaletteConfig {
    #[serde(default = "default_max_results")]
    pub max_results: usize,

    #[serde(default = "default_search_engine")]
    pub search_engine: String,

    #[serde(default)]
    pub custom_search_url: Option<String>,

    #[serde(default)]
    pub providers: ProviderConfig,

    #[serde(default = "default_file_tool")]
    pub file_search_tool: String,

    #[serde(default = "default_true")]
    pub show_provider_tags: bool,
}

impl Default for CommandPaletteConfig {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            search_engine: default_search_engine(),
            custom_search_url: None,
            providers: ProviderConfig::default(),
            file_search_tool: default_file_tool(),
            show_provider_tags: true,
        }
    }
}

// ── Module ──────────────────────────────────────────────────────────────────

pub struct CommandPalette {
    running: bool,
    #[allow(dead_code)]
    config: CommandPaletteConfig,
}

impl CommandPalette {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("command-palette").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for CommandPalette {
    fn id(&self) -> &'static str {
        "command-palette"
    }

    fn name(&self) -> &'static str {
        "Command Palette"
    }

    fn description(&self) -> &'static str {
        "Spotlight-like search: apps, calculator, web search, system commands and more"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Alt], "Space"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Command Palette module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Command Palette module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Command Palette: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                info!("Command Palette GUI spawned (pid={})", child.id());
            }
            Err(e) => {
                warn!("Failed to spawn Command Palette GUI ({bin:?}): {e}");
            }
        }
        Ok(())
    }
}

fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-command-palette");
        if candidate.exists() {
            return candidate;
        }
    }
    "mpt-command-palette".into()
}
