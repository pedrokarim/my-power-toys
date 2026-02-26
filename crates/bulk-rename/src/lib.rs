pub mod config;
#[cfg(feature = "gui")]
pub mod gui;
mod renamer;

use anyhow::Result;
use config::BulkRenameConfig;
use mpt_common::config::load_module_config;
use mpt_common::module::PowerModule;
use tracing::{info, warn};

pub use renamer::{ListOptions, RenameOperation, RenameOptions, RenamePreview, Renamer};

pub struct BulkRename {
    running: bool,
    pub config: BulkRenameConfig,
}

impl BulkRename {
    pub fn new() -> Self {
        Self {
            running: false,
            config: load_module_config("bulk-rename").unwrap_or_default(),
        }
    }
}

impl Default for BulkRename {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for BulkRename {
    fn id(&self) -> &'static str {
        "bulk-rename"
    }

    fn name(&self) -> &'static str {
        "Bulk Rename"
    }

    fn description(&self) -> &'static str {
        "Rename files in bulk using regex patterns with live preview and undo"
    }

    fn start(&mut self) -> Result<()> {
        self.config = load_module_config("bulk-rename").unwrap_or_default();
        self.running = true;
        info!("Bulk Rename module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Bulk Rename module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Bulk Rename: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                info!("Bulk Rename GUI spawned (pid={})", child.id());
            }
            Err(e) => {
                warn!("Failed to spawn Bulk Rename GUI ({bin:?}): {e}");
            }
        }
        Ok(())
    }
}

/// Find the mpt-bulk-rename binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-bulk-rename");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-bulk-rename".into()
}
