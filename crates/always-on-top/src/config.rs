use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlwaysOnTopConfig {
    #[serde(default = "default_true")]
    pub show_border: bool,

    #[serde(default = "default_border_color")]
    pub border_color: String,

    #[serde(default = "default_border_opacity")]
    pub border_opacity: f32,

    #[serde(default = "default_border_thickness")]
    pub border_thickness: u32,

    #[serde(default = "default_true")]
    pub play_sound: bool,

    #[serde(default)]
    pub excluded_apps: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_border_color() -> String {
    "#0078D4".into()
}

fn default_border_opacity() -> f32 {
    0.8
}

fn default_border_thickness() -> u32 {
    3
}

impl Default for AlwaysOnTopConfig {
    fn default() -> Self {
        Self {
            show_border: true,
            border_color: default_border_color(),
            border_opacity: default_border_opacity(),
            border_thickness: default_border_thickness(),
            play_sound: true,
            excluded_apps: Vec::new(),
        }
    }
}

/// Parse a hex color string like "#0078D4" or "#FF0000" into (r, g, b).
pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_colors() {
        assert_eq!(parse_hex_color("#0078D4"), Some((0, 120, 212)));
        assert_eq!(parse_hex_color("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("00FF00"), Some((0, 255, 0)));
        assert_eq!(parse_hex_color("#ZZZ"), None);
    }

    #[test]
    fn default_config_roundtrips() {
        let cfg = AlwaysOnTopConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let _: AlwaysOnTopConfig = toml::from_str(&s).unwrap();
    }
}
