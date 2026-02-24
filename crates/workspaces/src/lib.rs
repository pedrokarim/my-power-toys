pub mod config;
#[cfg(feature = "gui")]
pub mod gui;
pub mod launcher;
pub mod x11;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::{info, warn};

pub struct Workspaces {
    running: bool,
    config: config::WorkspacesConfig,
}

impl Workspaces {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("workspaces").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &config::WorkspacesConfig {
        &self.config
    }

    pub fn save_config(&self) -> Result<()> {
        mpt_common::config::save_module_config("workspaces", &self.config)
    }
}

impl Default for Workspaces {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for Workspaces {
    fn id(&self) -> &'static str {
        "workspaces"
    }

    fn name(&self) -> &'static str {
        "Workspaces"
    }

    fn description(&self) -> &'static str {
        "Save and restore desktop app layouts with one click"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Ctrl], "W"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!(
            "Workspaces started with {} workspace(s)",
            self.config.workspaces.len()
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Workspaces stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    /// Super+Ctrl+W -> open editor
    fn on_hotkey(&mut self) -> Result<()> {
        info!("Workspaces: opening editor");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .arg("--editor")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => info!("Workspaces editor spawned (pid={})", child.id()),
            Err(e) => warn!("Failed to spawn Workspaces editor ({bin:?}): {e}"),
        }
        Ok(())
    }
}

/// Find the mpt-workspaces binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-workspaces");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-workspaces".into()
}
