pub mod characters;
pub mod config;
pub mod monitor;
#[cfg(feature = "gui")]
pub mod overlay;

use anyhow::Result;
use config::QuickAccentConfig;
use mpt_common::module::PowerModule;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tracing::info;

pub struct QuickAccent {
    running: bool,
    config: QuickAccentConfig,
    stop_flag: Arc<AtomicBool>,
    monitor_thread: Option<thread::JoinHandle<()>>,
}

impl QuickAccent {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("quick-accent").unwrap_or_default();
        Self {
            running: false,
            config,
            stop_flag: Arc::new(AtomicBool::new(false)),
            monitor_thread: None,
        }
    }
}

impl Default for QuickAccent {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for QuickAccent {
    fn id(&self) -> &'static str {
        "quick-accent"
    }

    fn name(&self) -> &'static str {
        "Quick Accent"
    }

    fn description(&self) -> &'static str {
        "Type accented characters by holding a base letter and pressing an activation key"
    }

    fn default_hotkey(&self) -> Option<mpt_common::hotkey::Hotkey> {
        None
    }

    fn start(&mut self) -> Result<()> {
        info!("Quick Accent: starting");
        self.config = mpt_common::config::load_module_config("quick-accent").unwrap_or_default();
        self.stop_flag.store(false, Ordering::Relaxed);

        let stop = self.stop_flag.clone();
        let config = self.config.clone();
        let handle = monitor::spawn_monitor(config, stop);
        self.monitor_thread = Some(handle);

        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Quick Accent: stopping");
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(handle) = self.monitor_thread.take() {
            let _ = handle.join();
        }

        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

pub fn find_gui_binary() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("mpt-quick-accent");
        if candidate.exists() {
            return candidate;
        }
    }
    "mpt-quick-accent".into()
}
