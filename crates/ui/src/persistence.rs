use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::{ACCENT_THEMES, BUILTIN_BACKGROUNDS, GRADIENT_THEMES, VisualTheme};

#[derive(Serialize, Deserialize, Default)]
struct UiPrefs {
    #[serde(default)]
    visual_theme: String,
    #[serde(default)]
    custom_history: Vec<String>,
}

fn ui_prefs_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("my-power-toys")
        .join("ui.toml")
}

pub fn load_ui_prefs() -> (VisualTheme, Vec<PathBuf>) {
    let path = ui_prefs_path();
    let prefs: UiPrefs = std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default();

    let theme = parse_visual_theme(&prefs.visual_theme);
    let history = prefs.custom_history.iter().map(PathBuf::from).collect();
    (theme, history)
}

pub fn save_ui_prefs(theme: &VisualTheme, history: &[PathBuf]) {
    let prefs = UiPrefs {
        visual_theme: serialize_visual_theme(theme),
        custom_history: history
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    };
    if let Ok(s) = toml::to_string_pretty(&prefs) {
        let path = ui_prefs_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, s);
    }
}

fn parse_visual_theme(s: &str) -> VisualTheme {
    if s.is_empty() || s == "default" {
        return VisualTheme::Default;
    }
    if let Some(idx) = s.strip_prefix("color:")
        && let Ok(i) = idx.parse::<usize>()
        && i < ACCENT_THEMES.len()
    {
        return VisualTheme::Color(i);
    }
    if let Some(idx) = s.strip_prefix("gradient:")
        && let Ok(i) = idx.parse::<usize>()
        && i < GRADIENT_THEMES.len()
    {
        return VisualTheme::Gradient(i);
    }
    if let Some(idx) = s.strip_prefix("builtin:")
        && let Ok(i) = idx.parse::<usize>()
        && i < BUILTIN_BACKGROUNDS.len()
    {
        return VisualTheme::BuiltinImage(i);
    }
    if let Some(path) = s.strip_prefix("custom:") {
        return VisualTheme::CustomImage(PathBuf::from(path));
    }
    VisualTheme::Default
}

fn serialize_visual_theme(theme: &VisualTheme) -> String {
    match theme {
        VisualTheme::Default => "default".to_string(),
        VisualTheme::Color(i) => format!("color:{i}"),
        VisualTheme::Gradient(i) => format!("gradient:{i}"),
        VisualTheme::BuiltinImage(i) => format!("builtin:{i}"),
        VisualTheme::CustomImage(p) => format!("custom:{}", p.display()),
    }
}
