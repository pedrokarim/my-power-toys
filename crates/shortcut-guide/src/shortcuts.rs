use serde::{Deserialize, Serialize};

/// A shortcut entry for display in the guide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutEntry {
    pub keys: String,
    pub description: String,
    pub category: String,
}

/// Built-in GNOME shortcuts for the guide.
pub fn gnome_shortcuts() -> Vec<ShortcutEntry> {
    vec![
        ShortcutEntry {
            keys: "Super".into(),
            description: "Activities overview".into(),
            category: "System".into(),
        },
        ShortcutEntry {
            keys: "Super+L".into(),
            description: "Lock screen".into(),
            category: "System".into(),
        },
        ShortcutEntry {
            keys: "Super+D".into(),
            description: "Show desktop".into(),
            category: "System".into(),
        },
        ShortcutEntry {
            keys: "Super+A".into(),
            description: "Show all applications".into(),
            category: "System".into(),
        },
        ShortcutEntry {
            keys: "Super+Tab".into(),
            description: "Switch applications".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Alt+Tab".into(),
            description: "Switch windows".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Super+Left/Right".into(),
            description: "Tile window left/right".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Super+Up".into(),
            description: "Maximize window".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Super+Down".into(),
            description: "Restore / minimize window".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Alt+F4".into(),
            description: "Close window".into(),
            category: "Windows".into(),
        },
        ShortcutEntry {
            keys: "Ctrl+Alt+T".into(),
            description: "Open terminal".into(),
            category: "Apps".into(),
        },
        ShortcutEntry {
            keys: "Print".into(),
            description: "Screenshot".into(),
            category: "Apps".into(),
        },
        ShortcutEntry {
            keys: "Ctrl+Super+Up/Down".into(),
            description: "Switch workspace".into(),
            category: "Workspaces".into(),
        },
        ShortcutEntry {
            keys: "Ctrl+Super+Shift+Up/Down".into(),
            description: "Move window to workspace".into(),
            category: "Workspaces".into(),
        },
    ]
}

/// Group shortcuts by category.
pub fn group_by_category(shortcuts: &[ShortcutEntry]) -> Vec<(&str, Vec<&ShortcutEntry>)> {
    let mut categories: Vec<(&str, Vec<&ShortcutEntry>)> = Vec::new();
    for shortcut in shortcuts {
        if let Some(group) = categories
            .iter_mut()
            .find(|(cat, _)| *cat == shortcut.category)
        {
            group.1.push(shortcut);
        } else {
            categories.push((&shortcut.category, vec![shortcut]));
        }
    }
    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnome_shortcuts_not_empty() {
        let shortcuts = gnome_shortcuts();
        assert!(!shortcuts.is_empty());
        assert!(shortcuts.len() > 10);
    }

    #[test]
    fn group_categories() {
        let shortcuts = gnome_shortcuts();
        let groups = group_by_category(&shortcuts);
        assert!(groups.len() >= 3); // System, Windows, Apps, Workspaces
        let categories: Vec<&str> = groups.iter().map(|(c, _)| *c).collect();
        assert!(categories.contains(&"System"));
        assert!(categories.contains(&"Windows"));
    }

    #[test]
    fn serializable() {
        let shortcuts = gnome_shortcuts();
        let json = serde_json::to_string(&shortcuts).unwrap();
        assert!(json.contains("Super"));
        let parsed: Vec<ShortcutEntry> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), shortcuts.len());
    }
}
