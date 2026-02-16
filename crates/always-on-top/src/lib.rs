mod x11;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use mpt_common::platform::{self, DisplayServer};
use tracing::{info, warn};

pub struct AlwaysOnTop {
    running: bool,
}

impl AlwaysOnTop {
    pub fn new() -> Self {
        Self { running: false }
    }

    /// Toggle always-on-top for the currently focused window.
    pub fn toggle_active_window(&self) -> Result<()> {
        match platform::detect_display_server() {
            DisplayServer::X11 => x11::toggle_always_on_top(),
            DisplayServer::Wayland => {
                warn!("Wayland always-on-top requires compositor support (wlr-foreign-toplevel)");
                // Wayland implementation will come later — most compositors
                // don't expose this without a protocol extension.
                anyhow::bail!("Wayland support not yet implemented for always-on-top")
            }
            DisplayServer::Unknown => {
                anyhow::bail!("unknown display server, cannot toggle always-on-top")
            }
        }
    }
}

impl Default for AlwaysOnTop {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for AlwaysOnTop {
    fn id(&self) -> &'static str {
        "always-on-top"
    }

    fn name(&self) -> &'static str {
        "Always on Top"
    }

    fn description(&self) -> &'static str {
        "Pin the focused window to stay always on top with a keyboard shortcut"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super], "T"))
    }

    fn start(&mut self) -> Result<()> {
        info!("Always on Top module started");
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Always on Top module stopped");
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Always on Top hotkey triggered");
        self.toggle_active_window()
    }
}
