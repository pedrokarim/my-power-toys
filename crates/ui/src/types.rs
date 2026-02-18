use iced::font::Weight;
use iced::{Color, Font};
use std::path::PathBuf;

// ── Visual Theme ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisualTheme {
    Default,
    Color(usize),
    Gradient(usize),
    BuiltinImage(usize),
    CustomImage(PathBuf),
}

impl VisualTheme {
    pub fn is_glass(&self) -> bool {
        !matches!(self, VisualTheme::Default)
    }
}

/// (name, background RGB, preview/swatch RGB)
pub const ACCENT_THEMES: &[(&str, [u8; 3], [u8; 3])] = &[
    ("Midnight", [15, 15, 45], [60, 80, 180]),
    ("Forest", [10, 35, 15], [50, 150, 70]),
    ("Sunset", [50, 20, 10], [220, 120, 50]),
    ("Amethyst", [30, 10, 45], [160, 80, 200]),
    ("Ocean", [10, 30, 40], [50, 150, 200]),
    ("Rose", [45, 15, 25], [220, 80, 120]),
];

/// (name, angle_degrees, start RGB, mid RGB, end RGB)
pub type GradientTheme = (&'static str, f32, [u8; 3], [u8; 3], [u8; 3]);
pub const GRADIENT_THEMES: &[GradientTheme] = &[
    ("Aurora", 135.0, [10, 60, 30], [20, 40, 100], [60, 20, 100]),
    ("Sunset", 180.0, [120, 40, 20], [150, 40, 80], [70, 20, 100]),
    ("Ocean", 200.0, [5, 10, 30], [15, 45, 80], [5, 25, 50]),
    ("Boreal", 45.0, [15, 60, 45], [20, 75, 75], [45, 30, 75]),
];

/// (display name, filename in assets/backgrounds/)
pub const BUILTIN_BACKGROUNDS: &[(&str, &str)] = &[
    ("Space Station", "headquarter.jpg"),
    ("Colony", "colony.jpg"),
    ("Sandstorm", "storm.jpg"),
    ("Mountains", "concept-city.jpg"),
    ("Sacred Tree", "world-of-orio.jpg"),
    ("Pink Horizon", "home.jpg"),
    ("Crystal Ruins", "new-world.jpg"),
    ("Snow Peaks", "concept-bridge.jpg"),
    ("Orbital Dock", "fractal.jpg"),
    ("Street Life", "ethereal.jpg"),
    ("Orbital Ring", "cyberpunk.jpg"),
];

pub fn backgrounds_dir() -> PathBuf {
    if let Some(data) = dirs::data_dir() {
        let user_dir = data.join("my-power-toys").join("backgrounds");
        if user_dir.is_dir() {
            return user_dir;
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets")
        .join("backgrounds")
}

pub fn thumbnails_dir() -> PathBuf {
    backgrounds_dir().join("thumbnails")
}

// ── Data models ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: iced_fonts::bootstrap::Bootstrap,
    pub accent: Color,
    pub hotkey: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Module(String),
    Preferences,
    Tests,
    About,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Medium,
    Large,
}

impl FontSize {
    pub fn scale(self) -> f32 {
        match self {
            FontSize::Small => 0.9,
            FontSize::Medium => 1.0,
            FontSize::Large => 1.15,
        }
    }
}

/// UI context passed to widget builders for theme-aware colors and scaled sizes.
#[derive(Clone, Copy)]
pub struct Ui {
    pub dark: bool,
    pub s: f32,
    pub bold: bool,
    pub compact: bool,
    pub contrast: bool,
    pub glass: bool,
}

impl Ui {
    pub fn sz(&self, base: f32) -> f32 {
        (base * self.s).round()
    }

    pub fn font(&self) -> Font {
        if self.bold { bold() } else { Font::DEFAULT }
    }

    pub fn pad(&self, base: f32) -> f32 {
        if self.compact {
            (base * 0.7).round()
        } else {
            base
        }
    }

    /// High-contrast heading color for glass mode (white), theme default otherwise.
    pub fn heading(&self) -> Color {
        if self.glass {
            Color::WHITE
        } else if self.dark {
            Color::from_rgb8(205, 214, 244)
        } else {
            Color::from_rgb8(30, 30, 46)
        }
    }
}

pub fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..Font::DEFAULT
    }
}
