use serde::{Deserialize, Serialize};

/// Awake operating mode, mirroring Microsoft PowerToys Awake modes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AwakeMode {
    /// Module enabled but not keeping awake (passive).
    Off,
    /// Keep awake forever until manually stopped.
    #[default]
    Indefinite,
    /// Keep awake for a fixed duration (hours + minutes).
    Timed,
    /// Keep awake until a specific date/time.
    Expirable,
}

/// Persisted configuration for the Awake module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwakeConfig {
    #[serde(default)]
    pub mode: AwakeMode,

    /// When true, also prevent the display from turning off.
    /// When false, only prevent system suspend.
    #[serde(default = "default_true")]
    pub keep_screen_on: bool,

    /// Hours component for `Timed` mode.
    #[serde(default)]
    pub timed_hours: u32,

    /// Minutes component for `Timed` mode.
    #[serde(default = "default_30")]
    pub timed_minutes: u32,

    /// Expiration date-time string for `Expirable` mode.
    /// Format: `%Y-%m-%dT%H:%M` (e.g. `2026-03-01T18:00`).
    #[serde(default)]
    pub expire_at: String,
}

fn default_true() -> bool {
    true
}

fn default_30() -> u32 {
    30
}

impl Default for AwakeConfig {
    fn default() -> Self {
        Self {
            mode: AwakeMode::default(),
            keep_screen_on: true,
            timed_hours: 0,
            timed_minutes: default_30(),
            expire_at: String::new(),
        }
    }
}
