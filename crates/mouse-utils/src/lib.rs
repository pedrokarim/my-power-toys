pub mod config;
pub mod crosshairs;
pub mod cursor_wrap;
pub mod find_my_mouse;
pub mod highlighter;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;

use anyhow::Result;
use mpt_common::module::PowerModule;
use tracing::info;

pub struct MouseUtils {
    running: bool,
    config: config::MouseUtilsConfig,
    stop_flags: Vec<Arc<AtomicBool>>,
    threads: Vec<thread::JoinHandle<()>>,
}

impl MouseUtils {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("mouse-utils").unwrap_or_default();
        Self {
            running: false,
            config,
            stop_flags: Vec::new(),
            threads: Vec::new(),
        }
    }

    pub fn config(&self) -> &config::MouseUtilsConfig {
        &self.config
    }
}

impl Default for MouseUtils {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for MouseUtils {
    fn id(&self) -> &'static str {
        "mouse-utils"
    }

    fn name(&self) -> &'static str {
        "Mouse Utilities"
    }

    fn description(&self) -> &'static str {
        "Find My Mouse, click highlighter, crosshairs, mouse jump, cursor wrap, gliding cursor"
    }

    fn start(&mut self) -> Result<()> {
        // Reload config in case settings changed
        self.config = mpt_common::config::load_module_config("mouse-utils").unwrap_or_default();

        info!(
            "Mouse Utilities starting (find_mouse={}, highlighter={}, crosshair={}, jump={}, wrap={}, gliding={})",
            self.config.find_my_mouse,
            self.config.click_highlighter,
            self.config.crosshair,
            self.config.mouse_jump,
            self.config.cursor_wrap,
            self.config.gliding_cursor,
        );

        // Find My Mouse
        if self.config.find_my_mouse {
            let stop = Arc::new(AtomicBool::new(false));
            let handle = find_my_mouse::spawn(self.config.clone(), stop.clone());
            self.stop_flags.push(stop);
            self.threads.push(handle);
        }

        // Mouse Highlighter
        if self.config.click_highlighter {
            let stop = Arc::new(AtomicBool::new(false));
            let handle = highlighter::spawn(self.config.clone(), stop.clone());
            self.stop_flags.push(stop);
            self.threads.push(handle);
        }

        // Crosshairs
        if self.config.crosshair {
            let stop = Arc::new(AtomicBool::new(false));
            let handle = crosshairs::spawn(self.config.clone(), stop.clone());
            self.stop_flags.push(stop);
            self.threads.push(handle);
        }

        // Cursor Wrap
        if self.config.cursor_wrap {
            let stop = Arc::new(AtomicBool::new(false));
            let handle = cursor_wrap::spawn(stop.clone());
            self.stop_flags.push(stop);
            self.threads.push(handle);
        }

        // Mouse Jump & Gliding Cursor are more complex GUI features;
        // they will be implemented as separate overlay popups in a future phase.
        if self.config.mouse_jump {
            info!("Mouse Jump: enabled (popup overlay not yet implemented)");
        }
        if self.config.gliding_cursor {
            info!("Gliding Cursor: enabled (guided line overlay not yet implemented)");
        }

        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Mouse Utilities stopping...");

        // Signal all threads to stop
        for flag in &self.stop_flags {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }

        // Join all threads
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
        self.stop_flags.clear();

        self.running = false;
        info!("Mouse Utilities stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Mouse Utilities: hotkey triggered");
        Ok(())
    }
}
