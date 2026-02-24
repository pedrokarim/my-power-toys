pub mod config;
#[cfg(feature = "gui")]
pub mod editor;
pub mod layout;
#[cfg(feature = "gui")]
pub mod overlay;
pub mod x11;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use tracing::{info, warn};

pub struct FancyZones {
    running: bool,
    config: config::FancyZonesConfig,
}

impl FancyZones {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("fancy-zones").unwrap_or_default();
        Self {
            running: false,
            config,
        }
    }

    pub fn config(&self) -> &config::FancyZonesConfig {
        &self.config
    }

    /// Save current config to disk.
    pub fn save_config(&self) -> Result<()> {
        mpt_common::config::save_module_config("fancy-zones", &self.config)
    }
}

impl Default for FancyZones {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for FancyZones {
    fn id(&self) -> &'static str {
        "fancy-zones"
    }

    fn name(&self) -> &'static str {
        "FancyZones"
    }

    fn description(&self) -> &'static str {
        "Advanced window tiling with custom zone layouts"
    }

    /// Default hotkey opens the editor (Super+Shift+Z).
    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "Z"))
    }

    /// Additional hotkey: Super+Z opens the snap overlay.
    fn additional_hotkeys(&self) -> Vec<(Hotkey, &'static str)> {
        vec![(Hotkey::new(vec![Modifier::Super], "Z"), "snap")]
    }

    fn start(&mut self) -> Result<()> {
        self.running = true;
        info!(
            "FancyZones started with {} layout(s)",
            self.config.layouts.len()
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("FancyZones stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    /// Super+Shift+Z -> open editor
    fn on_hotkey(&mut self) -> Result<()> {
        info!("FancyZones: opening editor");

        let bin = find_gui_binary();
        let result = std::process::Command::new(&bin)
            .arg("--editor")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(child) => info!("FancyZones editor spawned (pid={})", child.id()),
            Err(e) => warn!("Failed to spawn FancyZones editor ({bin:?}): {e}"),
        }
        Ok(())
    }

    /// Super+Z -> open snap overlay
    fn on_named_action(&mut self, action: &str) -> Result<()> {
        if action == "snap" {
            info!("FancyZones: opening snap overlay");

            // Capture focused window BEFORE spawning overlay (overlay steals focus)
            let window_id = x11::get_focused_window()?;

            let bin = find_gui_binary();
            let result = std::process::Command::new(&bin)
                .arg("--window-id")
                .arg(window_id.to_string())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            match result {
                Ok(child) => info!("FancyZones overlay spawned (pid={})", child.id()),
                Err(e) => warn!("Failed to spawn FancyZones overlay ({bin:?}): {e}"),
            }
        } else {
            warn!("FancyZones: unknown action '{action}'");
        }
        Ok(())
    }
}

/// Find the mpt-fancy-zones binary, looking next to the current executable first.
fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-fancy-zones");
        if candidate.exists() {
            return candidate;
        }
    }
    // Fallback: rely on PATH
    "mpt-fancy-zones".into()
}
