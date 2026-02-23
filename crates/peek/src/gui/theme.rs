use egui::Color32;
use mpt_common::theme::GlassTheme;

const GLASS: GlassTheme = GlassTheme::dark();

// ── Glassmorphism backgrounds (from shared theme) ──────────────────────────
pub const BG_PRIMARY: Color32 = GLASS.bg_primary;
pub const BG_HEADER: Color32 = GLASS.bg_header;
pub const BG_FOOTER: Color32 = GLASS.bg_footer;
pub const SEPARATOR: Color32 = GLASS.separator;
pub const BORDER: Color32 = GLASS.border;

// ── Text (from shared theme) ───────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = GLASS.text_primary;
pub const TEXT_SECONDARY: Color32 = GLASS.text_secondary;
pub const TEXT_HINT: Color32 = GLASS.text_muted;

// ── Accent (from shared theme) ─────────────────────────────────────────────
pub const ACCENT: Color32 = GLASS.accent;

// ── Module-specific dimensions ─────────────────────────────────────────────
pub const WINDOW_WIDTH: f32 = 600.0;
pub const WINDOW_HEIGHT: f32 = 460.0;
pub const HEADER_HEIGHT: f32 = 52.0;
pub const FOOTER_HEIGHT: f32 = 40.0;
pub const CORNER_RADIUS: f32 = 12.0;
pub const INNER_PADDING: f32 = 16.0;

// ── Module-specific font sizes ─────────────────────────────────────────────
pub const FONT_FILENAME: f32 = 15.0;
pub const FONT_META: f32 = 11.0;
pub const FONT_CODE: f32 = 13.0;
pub const FONT_BODY: f32 = 13.0;
pub const FONT_BUTTON: f32 = 12.0;
pub const FONT_NAV: f32 = 16.0;
