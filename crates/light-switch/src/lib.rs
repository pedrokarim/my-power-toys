pub mod config;
mod solar;
mod theme;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveTime, Timelike};
use config::LightSwitchConfig;
use mpt_common::hotkey::{Hotkey, Modifier};
use mpt_common::module::PowerModule;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use theme::SystemTheme;
use tracing::info;

pub struct LightSwitch {
    running: bool,
    config: LightSwitchConfig,
    stop_flag: Arc<AtomicBool>,
    scheduler: Option<thread::JoinHandle<()>>,
}

impl LightSwitch {
    pub fn new() -> Self {
        let config = mpt_common::config::load_module_config("light-switch").unwrap_or_default();
        Self {
            running: false,
            config,
            stop_flag: Arc::new(AtomicBool::new(false)),
            scheduler: None,
        }
    }

    fn should_be_dark(&self) -> bool {
        let now = Local::now();

        match self.config.schedule_mode.as_str() {
            "sunset-sunrise" => {
                let (sunrise, sunset) = solar::sunrise_sunset(
                    now.year(),
                    now.month(),
                    now.day(),
                    self.config.latitude,
                    self.config.longitude,
                );

                // Apply timezone offset: solar times are in UTC, convert to local
                let tz_offset_min = now.offset().local_minus_utc() / 60;
                let sunrise = offset_time(sunrise, tz_offset_min + self.config.sunrise_offset_min);
                let sunset = offset_time(sunset, tz_offset_min + self.config.sunset_offset_min);

                let now_time = now.time();
                // Dark when before sunrise or after sunset
                now_time < sunrise || now_time >= sunset
            }
            "fixed" => {
                let dark_time = parse_time(&self.config.dark_mode_time).unwrap_or(NaiveTime::from_hms_opt(20, 0, 0).unwrap());
                let light_time = parse_time(&self.config.light_mode_time).unwrap_or(NaiveTime::from_hms_opt(6, 0, 0).unwrap());
                let now_time = now.time();

                if light_time < dark_time {
                    // Normal case: light during day, dark at night
                    now_time < light_time || now_time >= dark_time
                } else {
                    // Inverted: dark_time < light_time (e.g. dark at 02:00, light at 10:00)
                    now_time >= dark_time && now_time < light_time
                }
            }
            _ => {
                // "off" — no schedule, keep current
                theme::get_current_theme() == SystemTheme::Dark
            }
        }
    }

    fn apply_scheduled_theme(&self) {
        if self.config.schedule_mode == "off" {
            return;
        }
        let dark = self.should_be_dark();
        theme::set_theme(dark, self.config.apply_system, self.config.apply_apps);
    }

    fn spawn_scheduler(&mut self) {
        if self.config.schedule_mode == "off" {
            return;
        }

        let stop = self.stop_flag.clone();
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            let mut last_dark: Option<bool> = None;

            while !stop.load(Ordering::Relaxed) {
                let ls = LightSwitch {
                    running: true,
                    config: config.clone(),
                    stop_flag: Arc::new(AtomicBool::new(false)),
                    scheduler: None,
                };
                let dark = ls.should_be_dark();

                if last_dark != Some(dark) {
                    theme::set_theme(dark, config.apply_system, config.apply_apps);
                    last_dark = Some(dark);
                }

                thread::sleep(Duration::from_secs(60));
            }
        });

        self.scheduler = Some(handle);
    }
}

impl Default for LightSwitch {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerModule for LightSwitch {
    fn id(&self) -> &'static str {
        "light-switch"
    }

    fn name(&self) -> &'static str {
        "Light Switch"
    }

    fn description(&self) -> &'static str {
        "Switch between light and dark mode on schedule or shortcut"
    }

    fn default_hotkey(&self) -> Option<Hotkey> {
        Some(Hotkey::new(vec![Modifier::Super, Modifier::Shift], "D"))
    }

    fn start(&mut self) -> Result<()> {
        info!("Light Switch: starting");
        self.config = mpt_common::config::load_module_config("light-switch").unwrap_or_default();
        self.stop_flag.store(false, Ordering::Relaxed);

        // Apply theme immediately based on schedule
        self.apply_scheduled_theme();

        // Start background scheduler
        self.spawn_scheduler();

        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        info!("Light Switch: stopping");
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(handle) = self.scheduler.take() {
            let _ = handle.join();
        }

        self.running = false;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn on_hotkey(&mut self) -> Result<()> {
        info!("Light Switch: toggling theme via hotkey");
        theme::toggle_theme(self.config.apply_system, self.config.apply_apps);
        Ok(())
    }
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

fn offset_time(time: NaiveTime, offset_minutes: i32) -> NaiveTime {
    let total_min = time.hour() as i32 * 60 + time.minute() as i32 + offset_minutes;
    let total_min = total_min.rem_euclid(1440);
    NaiveTime::from_hms_opt(total_min as u32 / 60, total_min as u32 % 60, 0)
        .unwrap_or(time)
}
