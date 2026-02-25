use egui::Color32;
use mpt_common::theme::Theme;

const THEME: Theme = Theme::dark();

// ── Backgrounds (from shared theme) ────────────────────────────────────────
pub const BG_PRIMARY: Color32 = THEME.bg_primary;
pub const BG_SECONDARY: Color32 = THEME.bg_secondary;
pub const BG_CARD: Color32 = THEME.bg_card;
pub const BG_HEADER: Color32 = THEME.bg_header;
pub const BG_HOVER: Color32 = THEME.bg_hover;
pub const BG_CHIP: Color32 = THEME.bg_chip;
pub const BG_CHIP_SELECTED: Color32 = THEME.bg_chip_selected;
pub const BG_BUTTON: Color32 = THEME.bg_button;
pub const BG_BUTTON_HOVER: Color32 = THEME.bg_button_hover;
pub const BG_SUCCESS: Color32 = THEME.bg_success;
pub const BG_ERROR: Color32 = THEME.bg_error;

// ── Text (from shared theme) ───────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = THEME.text_primary;
pub const TEXT_SECONDARY: Color32 = THEME.text_secondary;
pub const TEXT_MUTED: Color32 = THEME.text_muted;
pub const TEXT_ACCENT: Color32 = THEME.text_accent;
pub const TEXT_SUCCESS: Color32 = THEME.text_success;
pub const TEXT_ERROR: Color32 = THEME.text_error;

// ── Borders (from shared theme) ────────────────────────────────────────────
pub const SEPARATOR: Color32 = THEME.separator;
pub const CARD_BORDER: Color32 = THEME.card_border;

// ── Shared dimensions (from shared theme) ──────────────────────────────────
pub use mpt_common::theme::{
    BUTTON_HEIGHT, BUTTON_RADIUS, CARD_RADIUS, CHIP_HEIGHT, CHIP_RADIUS, FONT_BODY, FONT_BUTTON,
    FONT_CHIP, FONT_SECTION, FONT_SMALL, FONT_TITLE, INNER_MARGIN, SECTION_SPACING,
};

// ── Module-specific dimensions ─────────────────────────────────────────────
pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 650.0;
pub const APP_CARD_HEIGHT: f32 = 100.0;
pub const APP_CARD_EXPANDED_HEIGHT: f32 = 132.0;
pub const MONITOR_PREVIEW_HEIGHT: f32 = 140.0;
pub const WORKSPACE_CARD_HEIGHT: f32 = 80.0;

/// Apply the theme to an egui context.
pub fn setup_visuals(ctx: &egui::Context) {
    THEME.apply(ctx);
}
