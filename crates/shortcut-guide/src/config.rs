use crate::shortcuts::ShortcutEntry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutGuideConfig {
    /// Show the built-in GNOME shortcut list.
    #[serde(default = "default_true")]
    pub show_gnome_shortcuts: bool,

    /// Categories to hide from the overlay (matched case-insensitively).
    #[serde(default)]
    pub hidden_categories: Vec<String>,

    /// User-defined shortcuts appended to the built-in ones.
    #[serde(default)]
    pub custom_shortcuts: Vec<ShortcutEntry>,
}

fn default_true() -> bool {
    true
}

impl Default for ShortcutGuideConfig {
    fn default() -> Self {
        Self {
            show_gnome_shortcuts: true,
            hidden_categories: Vec::new(),
            custom_shortcuts: Vec::new(),
        }
    }
}

impl ShortcutGuideConfig {
    /// Resolve the final shortcut list: built-in (optional) + custom, minus hidden categories.
    pub fn resolve(&self) -> Vec<ShortcutEntry> {
        let mut out = Vec::new();
        if self.show_gnome_shortcuts {
            out.extend(crate::shortcuts::gnome_shortcuts());
        }
        out.extend(self.custom_shortcuts.iter().cloned());
        out.retain(|s| {
            !self
                .hidden_categories
                .iter()
                .any(|c| c.eq_ignore_ascii_case(&s.category))
        });
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_include_gnome_shortcuts() {
        let cfg = ShortcutGuideConfig::default();
        assert!(!cfg.resolve().is_empty());
    }

    #[test]
    fn hidden_category_filters_out() {
        let mut cfg = ShortcutGuideConfig::default();
        cfg.hidden_categories.push("Windows".into());
        let list = cfg.resolve();
        assert!(list.iter().all(|s| s.category != "Windows"));
    }

    #[test]
    fn custom_shortcut_appears() {
        let mut cfg = ShortcutGuideConfig {
            show_gnome_shortcuts: false,
            ..Default::default()
        };
        cfg.custom_shortcuts.push(ShortcutEntry {
            keys: "Ctrl+Alt+K".into(),
            description: "Custom action".into(),
            category: "Custom".into(),
        });
        let list = cfg.resolve();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].keys, "Ctrl+Alt+K");
    }
}
