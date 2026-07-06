pub mod config;
#[cfg(feature = "gui")]
pub mod gui;
pub mod shortcuts;

use anyhow::Result;
use config::ShortcutGuideConfig;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::{info, warn};

pub struct ShortcutGuide {
    running: bool,
    #[allow(dead_code)]
    config: ShortcutGuideConfig,
}

impl ShortcutGuide {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("shortcut-guide").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }
}

impl Default for ShortcutGuide {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for ShortcutGuide {
    fn id(&self) -> &'static str {
        "shortcut-guide"
    }

    fn name(&self) -> &'static str {
        "Shortcut Guide"
    }

    fn description(&self) -> &'static str {
        "Overlay showing available keyboard shortcuts"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super], "?"))
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!("Shortcut Guide module started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Shortcut Guide module stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Shortcut Guide: launching GUI");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => info!("Shortcut Guide GUI spawned (pid={})", child.id()),
            Err(e) => warn!("Failed to spawn Shortcut Guide GUI ({bin:?}): {e}"),
        }
        Ok(())
    }
}

fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-shortcut-guide");
        if candidate.exists() {
            return candidate;
        }
    }
    "mpt-shortcut-guide".into()
}
