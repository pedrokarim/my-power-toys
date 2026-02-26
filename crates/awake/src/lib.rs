pub mod config;
mod inhibitor;

use anyhow::Result;
use chrono::Local;
use config::{AwakeConfig, AwakeMode};
use inhibitor::ScreenSaverInhibitor;
use mpt_common::module::PowerModule;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use tracing::info;

pub struct Awake {
    running: bool,
    config: AwakeConfig,
    inhibitor: Option<ScreenSaverInhibitor>,
    stop_flag: Arc<AtomicBool>,
    timer_thread: Option<thread::JoinHandle<()>>,
}

impl Awake {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("awake").unwrap_or_default();
        Self {
            running: false,
            config,
            inhibitor: None,
            stop_flag: Arc::new(AtomicBool::new(false)),
            timer_thread: None,
        }
    }

    pub fn config(&self) -> &AwakeConfig {
        &self.config
    }

    /// Spawn a background thread that auto-releases the inhibitor after a duration.
    fn spawn_timed_thread(&mut self) {
        let total_secs =
            (self.config.timed_hours as u64 * 3600) + (self.config.timed_minutes as u64 * 60);
        if total_secs == 0 {
            info!("Awake: timed mode with 0 duration, skipping timer");
            return;
        }

        let stop = self.stop_flag.clone();
        let duration = Duration::from_secs(total_secs);

        let handle = thread::spawn(move || {
            let start = Instant::now();
            while !stop.load(Ordering::Relaxed) {
                if start.elapsed() >= duration {
                    info!("Awake: timed duration elapsed, signalling stop");
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
                thread::sleep(Duration::from_secs(10));
            }
        });

        self.timer_thread = Some(handle);
    }

    /// Spawn a background thread that auto-releases the inhibitor at a specific date/time.
    fn spawn_expirable_thread(&mut self) {
        let expire_at =
            match chrono::NaiveDateTime::parse_from_str(&self.config.expire_at, "%Y-%m-%dT%H:%M") {
                Ok(dt) => dt,
                Err(e) => {
                    info!(
                        "Awake: invalid expire_at format '{}': {e}",
                        self.config.expire_at
                    );
                    return;
                }
            };

        let stop = self.stop_flag.clone();

        let handle = thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let now = Local::now().naive_local();
                if now >= expire_at {
                    info!("Awake: expiration time reached, signalling stop");
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
                thread::sleep(Duration::from_secs(10));
            }
        });

        self.timer_thread = Some(handle);
    }

    /// Release internal resources (inhibitor + timer thread).
    fn release_resources(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(handle) = self.timer_thread.take() {
            let _ = handle.join();
        }

        if let Some(inhibitor) = self.inhibitor.take() {
            if let Err(e) = inhibitor.release() {
                tracing::warn!("Failed to release inhibitor: {e}");
            } else {
                info!("Awake: inhibitor released");
            }
        }
    }

    /// Check if the timer thread has signalled that the duration/expiration has elapsed.
    /// If so, release the inhibitor. Returns true if the module auto-stopped.
    pub fn check_expired(&mut self) -> bool {
        if !self.running {
            return false;
        }
        if self.stop_flag.load(Ordering::Relaxed) && self.timer_thread.is_some() {
            info!("Awake: auto-stopping (timer/expiration elapsed)");
            self.release_resources();
            self.running = false;
            return true;
        }
        false
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
        // Reload config from disk (UI may have changed settings)
        self.config = mpt_common::config::load_module_config("awake").unwrap_or_default();
        self.stop_flag = Arc::new(AtomicBool::new(false));

        match self.config.mode {
            AwakeMode::Off => {
                info!("Awake: starting in passive (off) mode — no inhibition");
            }
            AwakeMode::Indefinite => {
                info!("Awake: starting in indefinite mode");
                let inhibitor = ScreenSaverInhibitor::inhibit(self.config.keep_screen_on)?;
                self.inhibitor = Some(inhibitor);
            }
            AwakeMode::Timed => {
                info!(
                    "Awake: starting in timed mode ({}h {}min)",
                    self.config.timed_hours, self.config.timed_minutes
                );
                let inhibitor = ScreenSaverInhibitor::inhibit(self.config.keep_screen_on)?;
                self.inhibitor = Some(inhibitor);
                self.spawn_timed_thread();
            }
            AwakeMode::Expirable => {
                info!(
                    "Awake: starting in expirable mode (until {})",
                    self.config.expire_at
                );
                let inhibitor = ScreenSaverInhibitor::inhibit(self.config.keep_screen_on)?;
                self.inhibitor = Some(inhibitor);
                self.spawn_expirable_thread();
            }
        }

        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Awake: stopping");
        self.release_resources();
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
