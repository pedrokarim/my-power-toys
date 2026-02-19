use egui::Color32;

// ── Background ──────────────────────────────────────────────────────────────
pub const BG_PRIMARY: Color32 = Color32::from_rgb(30, 30, 36);
pub const BG_SELECTED: Color32 = Color32::from_rgb(55, 75, 130);
pub const BG_HOVER: Color32 = Color32::from_rgb(42, 42, 50);
pub const SEPARATOR: Color32 = Color32::from_rgb(50, 50, 60);

// ── Text ────────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 230, 235);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(140, 140, 155);
pub const TEXT_HINT: Color32 = Color32::from_rgb(90, 90, 105);
pub const TEXT_TAG: Color32 = Color32::from_rgb(100, 100, 115);

// ── Accent ──────────────────────────────────────────────────────────────────
pub const ACCENT: Color32 = Color32::from_rgb(100, 140, 230);

// ── Dimensions ──────────────────────────────────────────────────────────────
pub const WINDOW_WIDTH: f32 = 650.0;
pub const SEARCH_BAR_HEIGHT: f32 = 52.0;
pub const RESULT_ROW_HEIGHT: f32 = 48.0;
pub const MAX_VISIBLE_RESULTS: usize = 8;
pub const CORNER_RADIUS: f32 = 12.0;
pub const INNER_PADDING: f32 = 16.0;

// ── Font sizes ──────────────────────────────────────────────────────────────
pub const FONT_SEARCH: f32 = 18.0;
pub const FONT_TITLE: f32 = 14.0;
pub const FONT_SUBTITLE: f32 = 11.0;
pub const FONT_TAG: f32 = 10.0;
pub const FONT_ICON: f32 = 20.0;
