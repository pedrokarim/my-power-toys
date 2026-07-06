use egui::Color32;
use mpt_common::theme::{self, GlassTheme};

const GLASS: GlassTheme = GlassTheme::dark();

// ── Glassmorphism backgrounds (from shared theme) ──────────────────────────
pub const BG_PRIMARY: Color32 = GLASS.bg_primary;
pub const BG_HEADER: Color32 = GLASS.bg_header;
pub const BG_CARD: Color32 = GLASS.bg_hover;
pub const SEPARATOR: Color32 = GLASS.separator;
pub const BORDER: Color32 = GLASS.border;
pub const ACCENT: Color32 = GLASS.accent;

// ── Text (from shared theme) ───────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = GLASS.text_primary;
pub const TEXT_SECONDARY: Color32 = GLASS.text_secondary;
pub const TEXT_MUTED: Color32 = GLASS.text_muted;

// ── Key badge (derived from shared theme, not hard-coded) ──────────────────
pub const KEY_BG: Color32 = GLASS.bg_hover;
pub const KEY_BORDER: Color32 = GLASS.border;
pub const KEY_TEXT: Color32 = GLASS.text_primary;

// ── Dimensions (aligned with command-palette overlay) ──────────────────────
pub const WINDOW_MAX_WIDTH: f32 = 780.0;
pub const WINDOW_MAX_HEIGHT: f32 = 640.0;
pub const CORNER_RADIUS: f32 = 12.0;
pub const INNER_PADDING: f32 = 16.0;
pub const CATEGORY_SPACING: f32 = 14.0;
pub const ROW_HEIGHT: f32 = 26.0;

// ── Fonts (from shared theme) ──────────────────────────────────────────────
pub const FONT_TITLE: f32 = theme::FONT_TITLE;
pub const FONT_SUBTITLE: f32 = theme::FONT_SMALL;
pub const FONT_CATEGORY: f32 = theme::FONT_SECTION;
pub const FONT_DESC: f32 = theme::FONT_BODY;
pub const FONT_KEY: f32 = theme::FONT_CHIP;
pub const FONT_HINT: f32 = theme::FONT_SMALL;
