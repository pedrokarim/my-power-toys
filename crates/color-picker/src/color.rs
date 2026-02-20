use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorFormat {
    Hex,
    Rgb,
    Hsl,
    Hsv,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string like "#FF8000" or "FF8000".
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() != 6 {
            anyhow::bail!("invalid hex color: #{hex}");
        }
        let r = u8::from_str_radix(&hex[0..2], 16).context("invalid red component")?;
        let g = u8::from_str_radix(&hex[2..4], 16).context("invalid green component")?;
        let b = u8::from_str_radix(&hex[4..6], 16).context("invalid blue component")?;
        Ok(Self::new(r, g, b))
    }

    pub fn format(self, fmt: ColorFormat) -> String {
        match fmt {
            ColorFormat::Hex => format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b),
            ColorFormat::Rgb => format!("rgb({}, {}, {})", self.r, self.g, self.b),
            ColorFormat::Hsl => {
                let (h, s, l) = self.to_hsl();
                format!("hsl({h}, {s}%, {l}%)")
            }
            ColorFormat::Hsv => {
                let (h, s, v) = self.to_hsv();
                format!("hsv({h}, {s}%, {v}%)")
            }
        }
    }

    pub fn to_hsl(self) -> (u16, u8, u8) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        if (max - min).abs() < f64::EPSILON {
            return (0, 0, (l * 100.0) as u8);
        }

        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };

        let h = if (max - r).abs() < f64::EPSILON {
            let mut h = (g - b) / d;
            if g < b {
                h += 6.0;
            }
            h
        } else if (max - g).abs() < f64::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        ((h * 60.0) as u16, (s * 100.0) as u8, (l * 100.0) as u8)
    }

    pub fn to_hsv(self) -> (u16, u8, u8) {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;

        let v = max;

        if max.abs() < f64::EPSILON {
            return (0, 0, (v * 100.0) as u8);
        }

        let s = d / max;

        if d.abs() < f64::EPSILON {
            return (0, 0, (v * 100.0) as u8);
        }

        let h = if (max - r).abs() < f64::EPSILON {
            let mut h = (g - b) / d;
            if g < b {
                h += 6.0;
            }
            h
        } else if (max - g).abs() < f64::EPSILON {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };

        ((h * 60.0) as u16, (s * 100.0) as u8, (v * 100.0) as u8)
    }

    /// Generate `count` shade variations from light to dark.
    pub fn shades(self, count: usize) -> Vec<Color> {
        if count == 0 {
            return vec![];
        }
        if count == 1 {
            return vec![self];
        }

        let (h, s, _l) = self.to_hsl();
        let h_f = h as f64;
        let s_f = s as f64 / 100.0;

        (0..count)
            .map(|i| {
                let t = i as f64 / (count - 1) as f64;
                let l_f = 0.9 - t * 0.8; // 90% → 10%
                Self::from_hsl(h_f, s_f, l_f)
            })
            .collect()
    }

    pub fn from_hsl(h: f64, s: f64, l: f64) -> Self {
        if s.abs() < f64::EPSILON {
            let v = (l * 255.0).round() as u8;
            return Self::new(v, v, v);
        }

        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;

        let r = hue_to_rgb(p, q, h / 360.0 + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h / 360.0);
        let b = hue_to_rgb(p, q, h / 360.0 - 1.0 / 3.0);

        Self::new(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }

    #[cfg(feature = "gui")]
    pub fn to_egui_color32(self) -> egui::Color32 {
        egui::Color32::from_rgb(self.r, self.g, self.b)
    }
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_format() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.format(ColorFormat::Hex), "#FF8000");
    }

    #[test]
    fn rgb_format() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.format(ColorFormat::Rgb), "rgb(255, 128, 0)");
    }

    #[test]
    fn hsl_format() {
        let c = Color::new(255, 0, 0);
        assert_eq!(c.format(ColorFormat::Hsl), "hsl(0, 100%, 50%)");
    }

    #[test]
    fn hsv_format() {
        let c = Color::new(255, 0, 0);
        assert_eq!(c.format(ColorFormat::Hsv), "hsv(0, 100%, 100%)");
    }

    #[test]
    fn black_hsl() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.format(ColorFormat::Hsl), "hsl(0, 0%, 0%)");
    }

    #[test]
    fn white_hsl() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.format(ColorFormat::Hsl), "hsl(0, 0%, 100%)");
    }

    #[test]
    fn black_hsv() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.format(ColorFormat::Hsv), "hsv(0, 0%, 0%)");
    }

    #[test]
    fn white_hsv() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.format(ColorFormat::Hsv), "hsv(0, 0%, 100%)");
    }

    #[test]
    fn from_hex_valid() {
        let c = Color::from_hex("#FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn from_hex_no_hash() {
        let c = Color::from_hex("FF8000").unwrap();
        assert_eq!(c.r, 255);
    }

    #[test]
    fn from_hex_invalid() {
        assert!(Color::from_hex("GG0000").is_err());
        assert!(Color::from_hex("#FFF").is_err());
    }

    #[test]
    fn shades_count() {
        let c = Color::new(255, 0, 0);
        let shades = c.shades(12);
        assert_eq!(shades.len(), 12);
        // First shade should be light, last should be dark
        let (_, _, l_first) = shades[0].to_hsl();
        let (_, _, l_last) = shades[11].to_hsl();
        assert!(l_first > l_last);
    }

    #[test]
    fn shades_zero() {
        let c = Color::new(128, 128, 128);
        assert!(c.shades(0).is_empty());
    }

    #[test]
    fn shades_one() {
        let c = Color::new(128, 128, 128);
        assert_eq!(c.shades(1).len(), 1);
    }
}
