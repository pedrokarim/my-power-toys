use egui::Color32;

/// Color theme for MyPowerToys GUI windows.
///
/// Use `Theme::dark()` or `Theme::light()` to get a complete set of tokens.
/// Tool windows (Image Resizer, Color Picker, …) default to dark.
/// The main UI can switch between both.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    // ── Backgrounds ────────────────────────────────────────────────────
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_card: Color32,
    pub bg_header: Color32,
    pub bg_hover: Color32,
    pub bg_chip: Color32,
    pub bg_chip_selected: Color32,
    pub bg_button: Color32,
    pub bg_button_hover: Color32,
    pub bg_success: Color32,
    pub bg_error: Color32,
    pub bg_progress: Color32,

    // ── Text ───────────────────────────────────────────────────────────
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_accent: Color32,
    pub text_success: Color32,
    pub text_error: Color32,

    // ── Borders / separators ───────────────────────────────────────────
    pub separator: Color32,
    pub card_border: Color32,
    pub drop_zone_border: Color32,
    pub drop_zone_border_active: Color32,
}

impl Theme {
    /// Dark theme — default for all tool windows.
    pub const fn dark() -> Self {
        Self {
            bg_primary: Color32::from_rgb(24, 24, 30),
            bg_secondary: Color32::from_rgb(32, 32, 40),
            bg_card: Color32::from_rgb(38, 38, 48),
            bg_header: Color32::from_rgb(20, 20, 26),
            bg_hover: Color32::from_rgb(50, 50, 62),
            bg_chip: Color32::from_rgb(45, 45, 56),
            bg_chip_selected: Color32::from_rgb(55, 75, 140),
            bg_button: Color32::from_rgb(55, 75, 140),
            bg_button_hover: Color32::from_rgb(70, 95, 170),
            bg_success: Color32::from_rgb(45, 140, 80),
            bg_error: Color32::from_rgb(180, 60, 60),
            bg_progress: Color32::from_rgb(55, 75, 140),

            text_primary: Color32::from_rgb(235, 235, 240),
            text_secondary: Color32::from_rgb(150, 150, 168),
            text_muted: Color32::from_rgb(90, 90, 108),
            text_accent: Color32::from_rgb(130, 160, 240),
            text_success: Color32::from_rgb(100, 210, 130),
            text_error: Color32::from_rgb(240, 100, 100),

            separator: Color32::from_rgb(44, 44, 54),
            card_border: Color32::from_rgb(50, 50, 62),
            drop_zone_border: Color32::from_rgb(70, 90, 150),
            drop_zone_border_active: Color32::from_rgb(100, 130, 210),
        }
    }

    /// Light theme — for the main settings UI.
    pub const fn light() -> Self {
        Self {
            bg_primary: Color32::from_rgb(248, 248, 252),
            bg_secondary: Color32::from_rgb(238, 238, 244),
            bg_card: Color32::from_rgb(255, 255, 255),
            bg_header: Color32::from_rgb(242, 242, 248),
            bg_hover: Color32::from_rgb(228, 228, 236),
            bg_chip: Color32::from_rgb(232, 232, 240),
            bg_chip_selected: Color32::from_rgb(55, 75, 140),
            bg_button: Color32::from_rgb(55, 75, 140),
            bg_button_hover: Color32::from_rgb(70, 95, 170),
            bg_success: Color32::from_rgb(220, 252, 231),
            bg_error: Color32::from_rgb(254, 226, 226),
            bg_progress: Color32::from_rgb(55, 75, 140),

            text_primary: Color32::from_rgb(28, 28, 36),
            text_secondary: Color32::from_rgb(100, 100, 118),
            text_muted: Color32::from_rgb(160, 160, 176),
            text_accent: Color32::from_rgb(45, 70, 180),
            text_success: Color32::from_rgb(22, 120, 55),
            text_error: Color32::from_rgb(200, 40, 40),

            separator: Color32::from_rgb(220, 220, 230),
            card_border: Color32::from_rgb(210, 210, 222),
            drop_zone_border: Color32::from_rgb(160, 170, 210),
            drop_zone_border_active: Color32::from_rgb(80, 110, 200),
        }
    }

    /// Apply this theme to an egui context via `set_visuals`.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut vis = egui::Visuals::dark();
        if self.is_light() {
            vis = egui::Visuals::light();
        }
        vis.window_fill = self.bg_primary;
        vis.panel_fill = self.bg_primary;
        vis.window_shadow = egui::epaint::Shadow::NONE;
        vis.window_stroke = egui::Stroke::NONE;
        vis.widgets.noninteractive.bg_fill = self.bg_primary;
        vis.widgets.inactive.bg_fill = self.bg_secondary;
        vis.widgets.hovered.bg_fill = self.bg_hover;
        vis.widgets.active.bg_fill = self.bg_button;
        ctx.set_visuals(vis);
    }

    /// Returns true if this is the light theme (bg_primary is bright).
    fn is_light(&self) -> bool {
        let [r, g, b, _] = self.bg_primary.to_array();
        (r as u16 + g as u16 + b as u16) > 384
    }
}

/// Glassmorphism overlay theme — for floating overlay windows (Command Palette, Peek).
/// Uses semi-transparent backgrounds. Text and accent colors come from the dark theme.
#[derive(Debug, Clone, Copy)]
pub struct GlassTheme {
    pub bg_primary: Color32,
    pub bg_header: Color32,
    pub bg_footer: Color32,
    pub bg_selected: Color32,
    pub bg_hover: Color32,
    pub separator: Color32,
    pub border: Color32,
    pub accent: Color32,

    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
}

impl GlassTheme {
    pub const fn dark() -> Self {
        let dark = Theme::dark();
        Self {
            bg_primary: Color32::from_rgba_premultiplied(25, 25, 35, 220),
            bg_header: Color32::from_rgba_premultiplied(30, 30, 42, 240),
            bg_footer: Color32::from_rgba_premultiplied(20, 20, 30, 240),
            bg_selected: Color32::from_rgba_premultiplied(55, 75, 135, 180),
            bg_hover: Color32::from_rgba_premultiplied(45, 45, 60, 160),
            separator: Color32::from_rgba_premultiplied(80, 80, 100, 60),
            border: Color32::from_rgba_premultiplied(90, 90, 120, 80),
            accent: dark.text_accent,

            text_primary: dark.text_primary,
            text_secondary: dark.text_secondary,
            text_muted: dark.text_muted,
        }
    }
}

// ── Shared dimensions ──────────────────────────────────────────────────────

pub const CARD_RADIUS: f32 = 8.0;
pub const CHIP_RADIUS: f32 = 6.0;
pub const CHIP_HEIGHT: f32 = 30.0;
pub const BUTTON_HEIGHT: f32 = 38.0;
pub const BUTTON_RADIUS: f32 = 8.0;
pub const SECTION_SPACING: f32 = 16.0;
pub const INNER_MARGIN: f32 = 20.0;

// ── Shared font sizes ──────────────────────────────────────────────────────

pub const FONT_TITLE: f32 = 18.0;
pub const FONT_BUTTON: f32 = 14.0;
pub const FONT_BODY: f32 = 13.0;
pub const FONT_CHIP: f32 = 12.0;
pub const FONT_SECTION: f32 = 12.0;
pub const FONT_SMALL: f32 = 11.5;
pub const FONT_ICON: f32 = 15.0;
