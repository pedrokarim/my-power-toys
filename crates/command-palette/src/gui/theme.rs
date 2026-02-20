use egui::Color32;

// ── Glassmorphism Background ───────────────────────────────────────────────
pub const BG_PRIMARY: Color32 = Color32::from_rgba_premultiplied(25, 25, 35, 220);
pub const BG_SELECTED: Color32 = Color32::from_rgba_premultiplied(55, 75, 135, 180);
pub const BG_HOVER: Color32 = Color32::from_rgba_premultiplied(45, 45, 60, 160);
pub const SEPARATOR: Color32 = Color32::from_rgba_premultiplied(80, 80, 100, 60);
pub const BORDER: Color32 = Color32::from_rgba_premultiplied(90, 90, 120, 80);

// ── Text ────────────────────────────────────────────────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 230, 235);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(150, 150, 168);
pub const TEXT_HINT: Color32 = Color32::from_rgb(100, 100, 120);
pub const TEXT_TAG: Color32 = Color32::from_rgb(110, 110, 130);

// ── Accent ──────────────────────────────────────────────────────────────────
pub const ACCENT: Color32 = Color32::from_rgb(110, 150, 240);

// ── Dimensions ──────────────────────────────────────────────────────────────
pub const WINDOW_WIDTH: f32 = 680.0;
pub const SEARCH_BAR_HEIGHT: f32 = 56.0;
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
