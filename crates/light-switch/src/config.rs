use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightSwitchConfig {
    #[serde(default = "default_off")]
    pub schedule_mode: String,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
    #[serde(default)]
    pub sunrise_offset_min: i32,
    #[serde(default)]
    pub sunset_offset_min: i32,
    #[serde(default = "default_dark_time")]
    pub dark_mode_time: String,
    #[serde(default = "default_light_time")]
    pub light_mode_time: String,
    #[serde(default = "default_true")]
    pub apply_system: bool,
    #[serde(default = "default_true")]
    pub apply_apps: bool,
}

fn default_off() -> String {
    "off".into()
}

fn default_dark_time() -> String {
    "20:00".into()
}

fn default_light_time() -> String {
    "06:00".into()
}

fn default_true() -> bool {
    true
}

impl Default for LightSwitchConfig {
    fn default() -> Self {
        Self {
            schedule_mode: default_off(),
            latitude: 0.0,
            longitude: 0.0,
            sunrise_offset_min: 0,
            sunset_offset_min: 0,
            dark_mode_time: default_dark_time(),
            light_mode_time: default_light_time(),
            apply_system: true,
            apply_apps: true,
        }
    }
}
