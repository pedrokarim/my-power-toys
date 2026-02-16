mod renamer;

use anyhow::Result;
use mpt_common::module::PowerModule;
use tracing::info;

pub use renamer::{RenameOperation, RenamePreview, Renamer};

pub struct BulkRename {
    running: bool,
}

impl BulkRename {
    pub fn new() -> Self {
        Self { running: false }
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
        info!("Bulk Rename: would open rename window");
        Ok(())
    }
}
