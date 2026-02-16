mod inhibitor;

use anyhow::Result;
use inhibitor::ScreenSaverInhibitor;
use mpt_common::module::PowerModule;
use tracing::info;

pub struct Awake {
    running: bool,
    inhibitor: Option<ScreenSaverInhibitor>,
}

impl Awake {
    pub fn new() -> Self {
        Self {
            running: false,
            inhibitor: None,
        }
    }
}

impl Default for Awake {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for Awake {
    fn id(&self) -> &'static str {
        "awake"
    }

    fn name(&self) -> &'static str {
        "Awake"
    }

    fn description(&self) -> &'static str {
        "Prevent the screen from going to sleep and the system from suspending"
    }

    fn start(&mut self) -> Result<()> {
        info!("Awake: inhibiting screensaver and suspend");
        let inhibitor = ScreenSaverInhibitor::inhibit()?;
        self.inhibitor = Some(inhibitor);
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(inhibitor) = self.inhibitor.take() {
            info!("Awake: releasing inhibitor");
            inhibitor.release()?;
        }
        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        if self.running {
            self.stop()
        } else {
            self.start()
        }
    }
}
