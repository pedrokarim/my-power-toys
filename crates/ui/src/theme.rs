//! Custom styles for MyPowerToys Settings UI.
//! Dark mode uses hand-picked ultra-dark colours instead of palette auto-gen.
//! When `glass` is true (image/gradient background active), surfaces become
//! semi-transparent for a frosted-glass look.

use iced::widget::{button, container};
use iced::{Border, Color, Shadow, Theme, Vector};

// ── Glass mode constants ───────────────────────────────────────────────

const GLASS_SIDEBAR: f32 = 0.82;
const GLASS_CARD: f32 = 0.72;
const GLASS_KBD: f32 = 0.78;

// ── True dark-mode backgrounds ─────────────────────────────────────────
// Hand-picked colours so dark mode feels properly dark, not washed-out.

/// Sidebar – deepest surface
const DARK_SIDEBAR: Color = Color {
    r: 0.035,
    g: 0.035,
    b: 0.055,
    a: 1.0,
}; // ≈ (9, 9, 14)

/// Cards & elevated surfaces
const DARK_SURFACE: Color = Color {
    r: 0.075,
    g: 0.075,
    b: 0.110,
    a: 1.0,
}; // ≈ (19, 19, 28)

/// Kbd badges, stat-card borders
const DARK_ELEVATED: Color = Color {
    r: 0.090,
    g: 0.090,
    b: 0.130,
    a: 1.0,
}; // ≈ (23, 23, 33)

/// Subtle borders
const DARK_BORDER: Color = Color {
    r: 0.12,
    g: 0.12,
    b: 0.16,
    a: 1.0,
}; // ≈ (31, 31, 41)

// ── Catppuccin accent colours (same across light/dark) ──────────────────

pub fn green() -> Color {
    Color::from_rgb8(166, 227, 161)
}
pub fn red() -> Color {
    Color::from_rgb8(243, 139, 168)
}
pub fn blue() -> Color {
    Color::from_rgb8(137, 180, 250)
}
pub fn mauve() -> Color {
    Color::from_rgb8(203, 166, 247)
}
pub fn pink() -> Color {
    Color::from_rgb8(245, 194, 231)
}
pub fn teal() -> Color {
    Color::from_rgb8(148, 226, 213)
}
pub fn yellow() -> Color {
    Color::from_rgb8(249, 226, 175)
}
pub fn peach() -> Color {
    Color::from_rgb8(250, 179, 135)
}
pub fn sky() -> Color {
    Color::from_rgb8(137, 220, 235)
}
pub fn lavender() -> Color {
    Color::from_rgb8(180, 190, 254)
}
pub fn flamingo() -> Color {
    Color::from_rgb8(242, 205, 205)
}
pub fn rosewater() -> Color {
    Color::from_rgb8(245, 224, 220)
}
pub fn maroon() -> Color {
    Color::from_rgb8(235, 160, 172)
}
pub fn sapphire() -> Color {
    Color::from_rgb8(116, 199, 236)
}

// ── Semantic colour helpers (adapt to dark / light) ─────────────────────

pub fn overlay0(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(80, 84, 104) // dimmer than Mocha
    } else {
        Color::from_rgb8(156, 160, 176)
    }
}
pub fn subtext0(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(140, 148, 175)
    } else {
        Color::from_rgb8(108, 111, 133)
    }
}
pub fn subtext1(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(170, 178, 208)
    } else {
        Color::from_rgb8(92, 95, 119)
    }
}

// ── Glass mode default text colour ───────────────────────────────────
/// Light text colour forced on all glass-mode containers so that labels,
/// toggle text, and any widget without an explicit `.color()` stay readable.
const GLASS_TEXT: Color = Color {
    r: 0.804,
    g: 0.839,
    b: 0.957,
    a: 1.0,
}; // ≈ (205, 214, 244) – same as dark-mode palette text

// ── Container styles ────────────────────────────────────────────────────

const NO_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

/// Dark sidebar background. Glass mode makes it semi-transparent.
pub fn sidebar(glass: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        let base = if glass || palette.is_dark {
            DARK_SIDEBAR
        } else {
            palette.background.strong.color
        };
        let bg = if glass {
            Color {
                a: GLASS_SIDEBAR,
                ..base
            }
        } else {
            base
        };
        container::Style {
            background: Some(bg.into()),
            border: Border::default(),
            shadow: NO_SHADOW,
            text_color: if glass { Some(GLASS_TEXT) } else { None },
        }
    }
}

/// Card with rounded corners. Glass mode makes it semi-transparent.
pub fn card(hc: bool, glass: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        let base = if glass || palette.is_dark {
            DARK_SURFACE
        } else {
            palette.background.weak.color
        };
        let bg = if glass {
            Color {
                a: GLASS_CARD,
                ..base
            }
        } else {
            base
        };
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: 12.0.into(),
                color: if hc {
                    if glass || palette.is_dark {
                        DARK_BORDER
                    } else {
                        palette.secondary.weak.color
                    }
                } else {
                    Color::TRANSPARENT
                },
                width: if hc { 1.5 } else { 0.0 },
            },
            shadow: NO_SHADOW,
            text_color: if glass { Some(GLASS_TEXT) } else { None },
        }
    }
}

/// Stat card with a thin border. Glass mode makes it semi-transparent.
pub fn stat_card(hc: bool, glass: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        let base = if glass || palette.is_dark {
            DARK_SURFACE
        } else {
            palette.background.weak.color
        };
        let border_color = if glass || palette.is_dark {
            DARK_BORDER
        } else {
            palette.background.strong.color
        };
        let bg = if glass {
            Color {
                a: GLASS_CARD,
                ..base
            }
        } else {
            base
        };
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: 10.0.into(),
                color: if glass {
                    Color {
                        a: 0.3,
                        ..border_color
                    }
                } else {
                    border_color
                },
                width: if hc { 2.0 } else { 1.0 },
            },
            shadow: NO_SHADOW,
            text_color: if glass { Some(GLASS_TEXT) } else { None },
        }
    }
}

/// Keyboard shortcut badge. Glass mode makes it semi-transparent.
pub fn kbd(hc: bool, glass: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        let base = if glass || palette.is_dark {
            DARK_ELEVATED
        } else {
            palette.background.strong.color
        };
        let bg = if glass {
            Color {
                a: GLASS_KBD,
                ..base
            }
        } else {
            base
        };
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: 6.0.into(),
                color: if glass || palette.is_dark {
                    DARK_BORDER
                } else {
                    palette.secondary.weak.color
                },
                width: if hc { 2.0 } else { 1.0 },
            },
            shadow: NO_SHADOW,
            text_color: if glass { Some(GLASS_TEXT) } else { None },
        }
    }
}

/// Colored icon badge (rounded square with accent background, like PowerToys).
pub fn icon_badge(accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let is_dark = theme.extended_palette().is_dark;
        let factor = if is_dark { 0.25 } else { 0.15 };
        let base = if is_dark { Color::BLACK } else { Color::WHITE };
        let bg = Color {
            r: accent.r * factor + base.r * (1.0 - factor),
            g: accent.g * factor + base.g * (1.0 - factor),
            b: accent.b * factor + base.b * (1.0 - factor),
            a: 1.0,
        };
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: 8.0.into(),
                color: Color::TRANSPARENT,
                width: 0.0,
            },
            shadow: NO_SHADOW,
            text_color: Some(accent),
        }
    }
}

// ── Button styles ───────────────────────────────────────────────────────

/// Navigation button in the sidebar.
pub fn nav_button(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let (bg, fg) = if selected {
            (
                Some(palette.primary.weak.color.into()),
                palette.primary.base.text,
            )
        } else {
            match status {
                button::Status::Hovered => {
                    let hover_bg = if palette.is_dark {
                        DARK_SURFACE
                    } else {
                        palette.background.weak.color
                    };
                    (Some(hover_bg.into()), palette.background.base.text)
                }
                _ => (None, palette.background.base.text),
            }
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                radius: 8.0.into(),
                color: Color::TRANSPARENT,
                width: 0.0,
            },
            shadow: NO_SHADOW,
        }
    }
}

/// Outer container for the segmented control.
pub fn segmented_control(glass: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        // Use DARK_ELEVATED (lighter than DARK_SURFACE) so it stands out from the card.
        let bg = if glass || palette.is_dark {
            DARK_ELEVATED
        } else {
            palette.background.weak.color
        };
        let bg = if glass {
            Color { a: 0.6, ..bg }
        } else {
            bg
        };
        container::Style {
            background: Some(bg.into()),
            border: Border {
                radius: 8.0.into(),
                color: if glass || palette.is_dark {
                    Color { a: 0.25, ..DARK_BORDER }
                } else {
                    Color::TRANSPARENT
                },
                width: if glass || palette.is_dark { 1.0 } else { 0.0 },
            },
            shadow: NO_SHADOW,
            text_color: None,
        }
    }
}

/// Individual segment button inside the segmented control.
pub fn seg_button(active: bool, glass: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let is_dark = glass || palette.is_dark;
        let (bg, fg) = if active {
            (
                Some(palette.primary.weak.color.into()),
                palette.primary.base.text,
            )
        } else {
            match status {
                button::Status::Hovered => {
                    let hover_bg = if is_dark {
                        Color::from_rgba8(255, 255, 255, 0.10)
                    } else {
                        palette.background.strong.color
                    };
                    let fg = if is_dark { GLASS_TEXT } else { palette.background.base.text };
                    (Some(hover_bg.into()), fg)
                }
                _ => {
                    // Key fix: use a visible light colour instead of palette.secondary.weak.text
                    let fg = if is_dark {
                        Color::from_rgb8(170, 178, 208)
                    } else {
                        palette.secondary.weak.text
                    };
                    (None, fg)
                }
            }
        };
        button::Style {
            background: bg,
            text_color: fg,
            border: Border {
                radius: 6.0.into(),
                color: Color::TRANSPARENT,
                width: 0.0,
            },
            shadow: NO_SHADOW,
        }
    }
}

// ── Theme swatch styles ─────────────────────────────────────────────────

/// Button style for a theme swatch in the picker.
pub fn swatch_btn(selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let bg = match status {
            button::Status::Hovered if !selected => {
                let hover = if palette.is_dark {
                    DARK_SURFACE
                } else {
                    palette.background.weak.color
                };
                Some(hover.into())
            }
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: palette.background.base.text,
            border: Border {
                radius: 10.0.into(),
                color: if selected {
                    blue()
                } else {
                    Color::TRANSPARENT
                },
                width: if selected { 2.5 } else { 0.0 },
            },
            shadow: NO_SHADOW,
        }
    }
}

/// Solid-color swatch preview container.
pub fn color_swatch(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(color.into()),
        border: Border {
            radius: 6.0.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        shadow: NO_SHADOW,
        text_color: None,
    }
}
