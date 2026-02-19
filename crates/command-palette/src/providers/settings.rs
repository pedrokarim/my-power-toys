use super::{PaletteResult, Provider, QueryContext, ResultAction, ResultIcon};

struct SettingsEntry {
    name: &'static str,
    subtitle: &'static str,
    command: &'static str,
}

const SETTINGS: &[SettingsEntry] = &[
    SettingsEntry {
        name: "MyPowerToys Settings",
        subtitle: "Open module settings",
        command: "mpt-settings",
    },
    SettingsEntry {
        name: "Display",
        subtitle: "Screen resolution, layout, scaling",
        command: "gnome-control-center display",
    },
    SettingsEntry {
        name: "Network",
        subtitle: "Wi-Fi, Ethernet, VPN",
        command: "gnome-control-center network",
    },
    SettingsEntry {
        name: "Sound",
        subtitle: "Volume, output, input devices",
        command: "gnome-control-center sound",
    },
    SettingsEntry {
        name: "Bluetooth",
        subtitle: "Paired devices, connections",
        command: "gnome-control-center bluetooth",
    },
    SettingsEntry {
        name: "Appearance",
        subtitle: "Theme, wallpaper, dark mode",
        command: "gnome-control-center appearance",
    },
    SettingsEntry {
        name: "Keyboard",
        subtitle: "Shortcuts, input sources, layout",
        command: "gnome-control-center keyboard",
    },
    SettingsEntry {
        name: "Power",
        subtitle: "Battery, suspend, screen blank",
        command: "gnome-control-center power",
    },
    SettingsEntry {
        name: "Users",
        subtitle: "User accounts, passwords",
        command: "gnome-control-center user-accounts",
    },
    SettingsEntry {
        name: "About",
        subtitle: "System info, OS version, hardware",
        command: "gnome-control-center info-overview",
    },
];

#[derive(Default)]
pub struct SettingsProvider;

impl SettingsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Provider for SettingsProvider {
    fn tag(&self) -> &'static str {
        "settings"
    }

    fn matches(&self, raw_query: &str) -> bool {
        raw_query.trim().starts_with('$')
    }

    fn strip_prefix<'a>(&self, raw_query: &'a str) -> &'a str {
        raw_query.trim().strip_prefix('$').unwrap_or("").trim()
    }

    fn search(&self, ctx: &QueryContext) -> Vec<PaletteResult> {
        let query = ctx.stripped_query.trim().to_lowercase();

        if query.is_empty() {
            return SETTINGS.iter().map(|e| to_result(e, 50.0)).collect();
        }

        SETTINGS
            .iter()
            .filter_map(|entry| {
                let name_lower = entry.name.to_lowercase();
                let sub_lower = entry.subtitle.to_lowercase();
                if name_lower == query {
                    Some(to_result(entry, 100.0))
                } else if name_lower.starts_with(&query) {
                    Some(to_result(entry, 80.0))
                } else if name_lower.contains(&query) || sub_lower.contains(&query) {
                    Some(to_result(entry, 60.0))
                } else {
                    None
                }
            })
            .collect()
    }
}

fn to_result(entry: &SettingsEntry, score: f64) -> PaletteResult {
    PaletteResult {
        id: format!("settings:{}", entry.name),
        title: entry.name.to_string(),
        subtitle: Some(entry.subtitle.to_string()),
        icon: ResultIcon::BuiltinSettings,
        action: ResultAction::OpenSettings(entry.command.to_string()),
        score,
        provider_tag: "settings",
    }
}
