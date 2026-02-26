pub mod config;

use anyhow::Result;
use mpt_common::module::PowerModule;
use tracing::info;

pub struct MouseUtils {
    running: bool,
    config: config::MouseUtilsConfig,
}

impl MouseUtils {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("mouse-utils").unwrap_or_default();
        Self {
            running: false,
            config,
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
        self.running = true;
        info!(
            "Mouse Utilities started (find_mouse={}, highlighter={}, crosshair={}, jump={}, wrap={}, gliding={})",
            self.config.find_my_mouse,
            self.config.click_highlighter,
            self.config.crosshair,
            self.config.mouse_jump,
            self.config.cursor_wrap,
            self.config.gliding_cursor,
        );
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        info!("Mouse Utilities stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        // Toggle find-my-mouse spotlight
        info!("Mouse Utilities: find my mouse spotlight triggered");
        Ok(())
    }
}
