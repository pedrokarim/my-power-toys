pub mod config;
#[cfg(feature = "gui")]
pub mod gui;
pub(crate) mod parser;

use anyhow::Result;
use config::HostsEditorConfig;
use mpt_common::config::load_module_config;
use mpt_common::module::PowerModule;
use tracing::{info, warn};

pub use parser::{HostEntry, HostsFile, HostsLine};

pub struct HostsEditor {
    running: bool,
    pub config: HostsEditorConfig,
}

impl HostsEditor {
    pub fn new() -> Self {
        Self {
            running: false,
            config: load_module_config("hosts-editor").unwrap_or_default(),
        }
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
        self.config = load_module_config("hosts-editor").unwrap_or_default();
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
        info!("Hosts Editor: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => {
                info!("Hosts Editor GUI spawned (pid={})", child.id());
            }
            Err(e) => {
                warn!("Failed to spawn Hosts Editor GUI ({bin:?}): {e}");
            }
        }
        Ok(())
    }
}

/// Find the mpt-hosts-editor binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-hosts-editor");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-hosts-editor".into()
}
