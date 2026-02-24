use crate::layout::Layout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FancyZonesConfig {
    /// Gap between zones in pixels.
    #[serde(default = "default_gap")]
    pub zone_gap: u32,

    /// Available layouts (the template palette).
    #[serde(default = "default_layouts")]
    pub layouts: Vec<Layout>,

    /// Per-monitor layout assignment: monitor name (e.g. "DP-1") -> index in `layouts`.
    #[serde(default)]
    pub monitor_layouts: HashMap<String, usize>,

    /// Default layout index used when a monitor has no specific assignment.
    #[serde(default)]
    pub default_layout: usize,

    /// Legacy field kept for backwards-compat deserialization.
    #[serde(default, skip_serializing)]
    pub active_layout: usize,
}

fn default_gap() -> u32 {
    8
}

fn default_layouts() -> Vec<Layout> {
    Layout::all_templates()
}

impl FancyZonesConfig {
    /// Get the layout assigned to a specific monitor, falling back to default.
    pub fn layout_for_monitor(&self, monitor_name: &str) -> Option<&Layout> {
        let idx = self
            .monitor_layouts
            .get(monitor_name)
            .copied()
            .unwrap_or(self.default_layout);
        self.layouts.get(idx)
    }
}

impl Default for FancyZonesConfig {
    fn default() -> Self {
        Self {
            zone_gap: default_gap(),
            layouts: default_layouts(),
            monitor_layouts: HashMap::new(),
            default_layout: 2, // "3 Columns" by default
            active_layout: 0,
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
        assert_eq!(parsed.zone_gap, 8);
        assert_eq!(parsed.layouts.len(), 6);
    }

    #[test]
    fn empty_toml_gives_defaults() {
        let parsed: FancyZonesConfig = toml::from_str("").unwrap();
        assert_eq!(parsed.zone_gap, 8);
        assert_eq!(parsed.layouts.len(), 6);
    }

    #[test]
    fn per_monitor_layout() {
        let mut config = FancyZonesConfig::default();
        config.monitor_layouts.insert("DP-1".to_string(), 4); // Grid
        config.monitor_layouts.insert("HDMI-1".to_string(), 1); // Focus

        let dp1 = config.layout_for_monitor("DP-1").unwrap();
        assert_eq!(dp1.name, "3x2 Grid");

        let hdmi = config.layout_for_monitor("HDMI-1").unwrap();
        assert_eq!(hdmi.name, "Focus");

        // Unknown monitor falls back to default_layout
        let unknown = config.layout_for_monitor("VGA-1").unwrap();
        assert_eq!(unknown.name, "3 Columns");
    }
}
