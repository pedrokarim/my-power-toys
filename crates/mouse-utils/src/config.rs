use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseUtilsConfig {
    /// Enable "Find My Mouse" — double-press Ctrl to spotlight the cursor.
    #[serde(default = "default_true")]
    pub find_my_mouse: bool,

    /// Enable click highlighter — show a circle on mouse clicks.
    #[serde(default)]
    pub click_highlighter: bool,

    /// Enable crosshair — permanent crosshair centered on cursor.
    #[serde(default)]
    pub crosshair: bool,

    /// Spotlight radius in pixels for find-my-mouse.
    #[serde(default = "default_radius")]
    pub spotlight_radius: u32,

    /// Spotlight color (hex).
    #[serde(default = "default_color")]
    pub spotlight_color: String,

    /// Spotlight opacity (0.0 - 1.0).
    #[serde(default = "default_opacity")]
    pub spotlight_opacity: f32,
}

fn default_true() -> bool {
    true
}

fn default_radius() -> u32 {
    100
}

fn default_color() -> String {
    "#FFFF00".to_string()
}

fn default_opacity() -> f32 {
    0.5
}

impl Default for MouseUtilsConfig {
    fn default() -> Self {
        Self {
            find_my_mouse: true,
            click_highlighter: false,
            crosshair: false,
            spotlight_radius: default_radius(),
            spotlight_color: default_color(),
            spotlight_opacity: default_opacity(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = MouseUtilsConfig::default();
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: MouseUtilsConfig = toml::from_str(&s).unwrap();
        assert!(parsed.find_my_mouse);
        assert!(!parsed.click_highlighter);
        assert_eq!(parsed.spotlight_radius, 100);
    }
}
