//! Custom styles for MyPowerToys Settings UI.
//! All container/button styles use the iced theme palette so they adapt
//! automatically to CatppuccinMocha (dark) and CatppuccinLatte (light).

use iced::widget::{button, container};
use iced::{Border, Color, Shadow, Theme, Vector};

// ── Catppuccin accent colours (same across light/dark) ──────────────────────

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

// ── Semantic colour helpers (adapt to dark / light) ─────────────────────────

pub fn overlay0(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(108, 112, 134) // Mocha overlay0
    } else {
        Color::from_rgb8(156, 160, 176) // Latte overlay0
    }
}
pub fn subtext0(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(166, 173, 200) // Mocha subtext0
    } else {
        Color::from_rgb8(108, 111, 133) // Latte subtext0
    }
}
pub fn subtext1(is_dark: bool) -> Color {
    if is_dark {
        Color::from_rgb8(186, 194, 222) // Mocha subtext1
    } else {
        Color::from_rgb8(92, 95, 119) // Latte subtext1
    }
}

// ── Container styles ────────────────────────────────────────────────────────

const NO_SHADOW: Shadow = Shadow {
    color: Color::TRANSPARENT,
    offset: Vector { x: 0.0, y: 0.0 },
    blur_radius: 0.0,
};

/// Dark sidebar background — uses the strongest background shade.
pub fn sidebar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.strong.color.into()),
        border: Border::default(),
        shadow: NO_SHADOW,
        text_color: None,
    }
}

/// Card with rounded corners. High-contrast adds a visible border.
pub fn card(hc: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                radius: 12.0.into(),
                color: if hc { palette.secondary.weak.color } else { Color::TRANSPARENT },
                width: if hc { 1.5 } else { 0.0 },
            },
            shadow: NO_SHADOW,
            text_color: None,
        }
    }
}

/// Stat card with a thin border. High-contrast makes it thicker.
pub fn stat_card(hc: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.weak.color.into()),
            border: Border {
                radius: 10.0.into(),
                color: palette.background.strong.color,
                width: if hc { 2.0 } else { 1.0 },
            },
            shadow: NO_SHADOW,
            text_color: None,
        }
    }
}

/// Keyboard shortcut badge. High-contrast makes the border thicker.
pub fn kbd(hc: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let palette = theme.extended_palette();
        container::Style {
            background: Some(palette.background.strong.color.into()),
            border: Border {
                radius: 6.0.into(),
                color: palette.secondary.weak.color,
                width: if hc { 2.0 } else { 1.0 },
            },
            shadow: NO_SHADOW,
            text_color: None,
        }
    }
}

/// Colored icon badge (rounded square with accent background, like PowerToys).
pub fn icon_badge(accent: Color) -> impl Fn(&Theme) -> container::Style {
    move |theme| {
        let is_dark = theme.extended_palette().is_dark;
        let factor = if is_dark { 0.25 } else { 0.15 };
        let base = if is_dark {
            Color::BLACK
        } else {
            Color::WHITE
        };
        // Blend accent with base at low opacity
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

// ── Button styles ───────────────────────────────────────────────────────────

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
                button::Status::Hovered => (
                    Some(palette.background.weak.color.into()),
                    palette.background.base.text,
                ),
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
pub fn segmented_control(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style {
        background: Some(palette.background.weak.color.into()),
        border: Border {
            radius: 8.0.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        shadow: NO_SHADOW,
        text_color: None,
    }
}

/// Individual segment button inside the segmented control.
pub fn seg_button(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| {
        let palette = theme.extended_palette();
        let (bg, fg) = if active {
            (
                Some(palette.primary.weak.color.into()),
                palette.primary.base.text,
            )
        } else {
            match status {
                button::Status::Hovered => (
                    Some(palette.background.strong.color.into()),
                    palette.background.base.text,
                ),
                _ => (None, palette.secondary.weak.text),
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

