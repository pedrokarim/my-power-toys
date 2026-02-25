pub mod border;
pub mod config;
mod x11;

use std::collections::HashSet;
use std::sync::mpsc;

use anyhow::Result;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use mpt_common::platform::{self, DisplayServer};
use tracing::{info, warn};

use border::BorderCmd;
use config::AlwaysOnTopConfig;

pub use config::AlwaysOnTopConfig as Config;

pub struct AlwaysOnTop {
    running: bool,
    config: AlwaysOnTopConfig,
    pinned: HashSet<u32>,
    border_tx: Option<mpsc::Sender<BorderCmd>>,
}

impl AlwaysOnTop {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("always-on-top").unwrap_or_default();
        Self {
            running: false,
            config,
            pinned: HashSet::new(),
            border_tx: None,
        }
    }

    fn toggle_active_window(&mut self) -> Result<()> {
        match platform::detect_display_server() {
            DisplayServer::X11 => self.toggle_x11(),
            DisplayServer::Wayland => {
                warn!("Wayland always-on-top requires compositor support (wlr-foreign-toplevel)");
                anyhow::bail!("Wayland support not yet implemented for always-on-top")
            }
            DisplayServer::Unknown => {
                anyhow::bail!("unknown display server, cannot toggle always-on-top")
            }
        }
    }

    fn toggle_x11(&mut self) -> Result<()> {
        let focused = x11::get_focused_window()?;

        // Check excluded apps
        if self.is_excluded(&focused.wm_class) {
            info!(
                "Window '{}' is excluded from always-on-top",
                focused.wm_class
            );
            return Ok(());
        }

        // Determine if we're pinning or unpinning based on our own state tracking.
        // We use self.pinned instead of the X11 _NET_WM_STATE_ABOVE property
        // because the WM may not have processed a prior toggle yet (race condition).
        let will_pin = !self.pinned.contains(&focused.id);

        // Explicitly add or remove the X11 state
        x11::set_always_on_top(focused.id, focused.root, will_pin)?;

        // Update tracking
        if will_pin {
            self.pinned.insert(focused.id);
            info!("Pinned window {} ({})", focused.id, focused.wm_class);
        } else {
            self.pinned.remove(&focused.id);
            info!("Unpinned window {} ({})", focused.id, focused.wm_class);
        }

        // Show/hide border
        if self.config.show_border
            && let Some(tx) = &self.border_tx
        {
            let cmd = if will_pin {
                BorderCmd::Add(focused.id)
            } else {
                BorderCmd::Remove(focused.id)
            };
            let _ = tx.send(cmd);
        }

        // Play sound
        if self.config.play_sound {
            play_notification_sound();
        }

        Ok(())
    }

    fn is_excluded(&self, wm_class: &str) -> bool {
        if wm_class.is_empty() {
            return false;
        }
        let lower = wm_class.to_lowercase();
        self.config
            .excluded_apps
            .iter()
            .any(|app| lower.contains(&app.to_lowercase()))
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

        // Start border thread if borders are enabled and we're on X11
        if self.config.show_border
            && matches!(platform::detect_display_server(), DisplayServer::X11)
        {
            match border::spawn_border_thread(&self.config) {
                Ok(tx) => self.border_tx = Some(tx),
                Err(e) => warn!("Failed to start border thread: {e}"),
            }
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Always on Top module stopped");
        self.running = false;

        // Shutdown border thread
        if let Some(tx) = self.border_tx.take() {
            let _ = tx.send(BorderCmd::Shutdown);
        }

        self.pinned.clear();
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

/// Play a short notification sound using the system's canberra-gtk-play.
fn play_notification_sound() {
    std::thread::spawn(|| {
        let result = std::process::Command::new("canberra-gtk-play")
            .args(["-i", "bell", "-d", "Always On Top"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        if let Err(e) = result {
            // canberra-gtk-play not installed — silently ignore
            tracing::debug!("canberra-gtk-play not available: {e}");
        }
    });
}
