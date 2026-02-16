mod parser;

use anyhow::Result;
use mpt_common::module::PowerModule;
use tracing::info;

pub use parser::{HostEntry, HostsFile};

pub struct HostsEditor {
    running: bool,
}

impl HostsEditor {
    pub fn new() -> Self {
        Self { running: false }
    }
}

impl Default for HostsEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for HostsEditor {
    fn id(&self) -> &'static str {
        "hosts-editor"
    }

    fn name(&self) -> &'static str {
        "Hosts Editor"
    }

    fn description(&self) -> &'static str {
        "Graphical editor for /etc/hosts with toggle entries on/off"
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Hosts Editor module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Hosts Editor module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        // Will open the GUI editor window in the future
        info!("Hosts Editor: would open editor window");
        Ok(())
    }
}
