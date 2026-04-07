use serde::{Deserialize, Serialize};

// ── Enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum FindMyMouseActivation {
    #[default]
    LeftCtrlTwice,
    RightCtrlTwice,
    ShakeMouse,
    CustomShortcut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum HighlightMode {
    #[default]
    CircleHighlight,
    Spotlight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CrosshairOrientation {
    Horizontal,
    Vertical,
    #[default]
    Both,
}

// ── Config ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseUtilsConfig {
    // ── Find My Mouse ──────────────────────────────────────────────────
    #[serde(default = "default_true")]
    pub find_my_mouse: bool,
    #[serde(default)]
    pub find_my_mouse_activation: FindMyMouseActivation,
    #[serde(default)]
    pub find_my_mouse_shortcut: String,
    #[serde(default = "default_shake_distance")]
    pub find_my_mouse_shake_distance: u32,
    #[serde(default = "default_true")]
    pub find_my_mouse_game_mode: bool,
    #[serde(default = "default_bg_color")]
    pub find_my_mouse_bg_color: String,
    #[serde(default = "default_spotlight_color")]
    pub find_my_mouse_spotlight_color: String,
    #[serde(default = "default_spotlight_radius")]
    pub find_my_mouse_spotlight_radius: u32,
    #[serde(default = "default_overlay_opacity")]
    pub find_my_mouse_overlay_opacity: f32,
    #[serde(default = "default_initial_zoom")]
    pub find_my_mouse_initial_zoom: u32,
    #[serde(default = "default_animation_ms")]
    pub find_my_mouse_animation_ms: u32,
    #[serde(default)]
    pub find_my_mouse_excluded_apps: String,

    // ── Mouse Highlighter ──────────────────────────────────────────────
    #[serde(default)]
    pub click_highlighter: bool,
    #[serde(default = "default_highlighter_shortcut")]
    pub highlighter_shortcut: String,
    #[serde(default = "default_highlighter_primary")]
    pub highlighter_primary_color: String,
    #[serde(default = "default_highlighter_secondary")]
    pub highlighter_secondary_color: String,
    #[serde(default)]
    pub highlighter_always_color: String,
    #[serde(default)]
    pub highlighter_mode: HighlightMode,
    #[serde(default = "default_highlighter_radius")]
    pub highlighter_radius: u32,
    #[serde(default = "default_fade_delay")]
    pub highlighter_fade_delay_ms: u32,
    #[serde(default = "default_fade_duration")]
    pub highlighter_fade_duration_ms: u32,

    // ── Crosshairs ─────────────────────────────────────────────────────
    #[serde(default)]
    pub crosshair: bool,
    #[serde(default)]
    pub crosshair_shortcut: String,
    #[serde(default = "default_crosshair_color")]
    pub crosshair_color: String,
    #[serde(default = "default_crosshair_opacity")]
    pub crosshair_opacity: f32,
    #[serde(default = "default_crosshair_center_radius")]
    pub crosshair_center_radius: u32,
    #[serde(default = "default_crosshair_thickness")]
    pub crosshair_thickness: u32,
    #[serde(default = "default_crosshair_border_color")]
    pub crosshair_border_color: String,
    #[serde(default)]
    pub crosshair_border_size: u32,
    #[serde(default)]
    pub crosshair_orientation: CrosshairOrientation,
    #[serde(default)]
    pub crosshair_auto_hide: bool,
    #[serde(default)]
    pub crosshair_fixed_length: bool,
    #[serde(default = "default_crosshair_fixed_length_px")]
    pub crosshair_fixed_length_px: u32,

    // ── Mouse Jump ─────────────────────────────────────────────────────
    #[serde(default)]
    pub mouse_jump: bool,
    #[serde(default = "default_jump_shortcut")]
    pub mouse_jump_shortcut: String,
    #[serde(default = "default_jump_width")]
    pub mouse_jump_max_width: u32,
    #[serde(default = "default_jump_height")]
    pub mouse_jump_max_height: u32,

    // ── Cursor Wrap ────────────────────────────────────────────────────
    #[serde(default)]
    pub cursor_wrap: bool,

    // ── Gliding Cursor ─────────────────────────────────────────────────
    #[serde(default)]
    pub gliding_cursor: bool,
    #[serde(default = "default_gliding_shortcut")]
    pub gliding_cursor_shortcut: String,
    #[serde(default = "default_gliding_travel")]
    pub gliding_cursor_travel_speed: u32,
    #[serde(default = "default_gliding_delay")]
    pub gliding_cursor_delay_speed: u32,
}

// ── Defaults ───────────────────────────────────────────────────────────

fn default_true() -> bool {
    true
}

// Find My Mouse
fn default_shake_distance() -> u32 {
    1000
}
fn default_bg_color() -> String {
    "#000000".into()
}
fn default_spotlight_color() -> String {
    "#FFFFFF".into()
}
fn default_spotlight_radius() -> u32 {
    100
}
fn default_overlay_opacity() -> f32 {
    0.5
}
fn default_initial_zoom() -> u32 {
    4
}
fn default_animation_ms() -> u32 {
    500
}

// Mouse Highlighter
fn default_highlighter_shortcut() -> String {
    "Super+Shift+H".into()
}
fn default_highlighter_primary() -> String {
    "#FFFF00".into()
}
fn default_highlighter_secondary() -> String {
    "#0000FF".into()
}
fn default_highlighter_radius() -> u32 {
    20
}
fn default_fade_delay() -> u32 {
    500
}
fn default_fade_duration() -> u32 {
    250
}

// Crosshairs
fn default_crosshair_color() -> String {
    "#FF0000".into()
}
fn default_crosshair_opacity() -> f32 {
    0.75
}
fn default_crosshair_center_radius() -> u32 {
    20
}
fn default_crosshair_thickness() -> u32 {
    5
}
fn default_crosshair_border_color() -> String {
    "#FFFFFF".into()
}
fn default_crosshair_fixed_length_px() -> u32 {
    400
}

// Mouse Jump
fn default_jump_shortcut() -> String {
    "Super+Shift+D".into()
}
fn default_jump_width() -> u32 {
    400
}
fn default_jump_height() -> u32 {
    300
}

// Gliding Cursor
fn default_gliding_shortcut() -> String {
    "Super+Alt+.".into()
}
fn default_gliding_travel() -> u32 {
    1000
}
fn default_gliding_delay() -> u32 {
    500
}

impl Default for MouseUtilsConfig {
    fn default() -> Self {
        Self {
            // Find My Mouse
            find_my_mouse: true,
            find_my_mouse_activation: FindMyMouseActivation::default(),
            find_my_mouse_shortcut: String::new(),
            find_my_mouse_shake_distance: default_shake_distance(),
            find_my_mouse_game_mode: true,
            find_my_mouse_bg_color: default_bg_color(),
            find_my_mouse_spotlight_color: default_spotlight_color(),
            find_my_mouse_spotlight_radius: default_spotlight_radius(),
            find_my_mouse_overlay_opacity: default_overlay_opacity(),
            find_my_mouse_initial_zoom: default_initial_zoom(),
            find_my_mouse_animation_ms: default_animation_ms(),
            find_my_mouse_excluded_apps: String::new(),

            // Mouse Highlighter
            click_highlighter: false,
            highlighter_shortcut: default_highlighter_shortcut(),
            highlighter_primary_color: default_highlighter_primary(),
            highlighter_secondary_color: default_highlighter_secondary(),
            highlighter_always_color: String::new(),
            highlighter_mode: HighlightMode::default(),
            highlighter_radius: default_highlighter_radius(),
            highlighter_fade_delay_ms: default_fade_delay(),
            highlighter_fade_duration_ms: default_fade_duration(),

            // Crosshairs
            crosshair: false,
            crosshair_shortcut: String::new(),
            crosshair_color: default_crosshair_color(),
            crosshair_opacity: default_crosshair_opacity(),
            crosshair_center_radius: default_crosshair_center_radius(),
            crosshair_thickness: default_crosshair_thickness(),
            crosshair_border_color: default_crosshair_border_color(),
            crosshair_border_size: 0,
            crosshair_orientation: CrosshairOrientation::default(),
            crosshair_auto_hide: false,
            crosshair_fixed_length: false,
            crosshair_fixed_length_px: default_crosshair_fixed_length_px(),

            // Mouse Jump
            mouse_jump: false,
            mouse_jump_shortcut: default_jump_shortcut(),
            mouse_jump_max_width: 400,
            mouse_jump_max_height: 300,

            // Cursor Wrap
            cursor_wrap: false,

            // Gliding Cursor
            gliding_cursor: false,
            gliding_cursor_shortcut: default_gliding_shortcut(),
            gliding_cursor_travel_speed: default_gliding_travel(),
            gliding_cursor_delay_speed: default_gliding_delay(),
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
        assert!(!parsed.crosshair);
        assert!(!parsed.mouse_jump);
        assert!(!parsed.cursor_wrap);
        assert!(!parsed.gliding_cursor);
        assert_eq!(parsed.find_my_mouse_spotlight_radius, 100);
        assert_eq!(parsed.find_my_mouse_animation_ms, 500);
        assert_eq!(parsed.highlighter_radius, 20);
        assert_eq!(parsed.crosshair_thickness, 5);
        assert_eq!(parsed.mouse_jump_max_width, 400);
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let toml_str = r#"
            find_my_mouse = false
            click_highlighter = true
        "#;
        let parsed: MouseUtilsConfig = toml::from_str(toml_str).unwrap();
        assert!(!parsed.find_my_mouse);
        assert!(parsed.click_highlighter);
        assert_eq!(parsed.find_my_mouse_spotlight_radius, 100);
        assert_eq!(parsed.crosshair_opacity, 0.75);
        assert!(!parsed.mouse_jump);
    }
}
