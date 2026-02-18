use crate::layout::Layout;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyZonesConfig {
    /// Index of the currently active layout (0-based).
    #[serde(default)]
    pub active_layout: usize,

    /// Gap between zones in pixels.
    #[serde(default = "default_gap")]
    pub zone_gap: u32,

    /// Available layouts. Defaults to 2-col, 3-col, and main+stack.
    #[serde(default = "default_layouts")]
    pub layouts: Vec<Layout>,
}

fn default_gap() -> u32 {
    8
}

fn default_layouts() -> Vec<Layout> {
    vec![
        Layout::default_columns(2),
        Layout::default_columns(3),
        Layout::main_plus_stack(),
    ]
}

impl Default for FancyZonesConfig {
    fn default() -> Self {
        Self {
            active_layout: 0,
            zone_gap: default_gap(),
            layouts: default_layouts(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = FancyZonesConfig::default();
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: FancyZonesConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.active_layout, 0);
        assert_eq!(parsed.zone_gap, 8);
        assert_eq!(parsed.layouts.len(), 3);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let parsed: FancyZonesConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.active_layout, 0);
        assert_eq!(parsed.zone_gap, 8);
        assert_eq!(parsed.layouts.len(), 3);
    }

    #[test]
    fn active_layout_persists() {
        let config = FancyZonesConfig {
            active_layout: 2,
            ..FancyZonesConfig::default()
        };
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: FancyZonesConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.active_layout, 2);
    }
}
