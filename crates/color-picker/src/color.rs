use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy)]
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
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn format(self, fmt: ColorFormat) -> String {
        match fmt {
            ColorFormat::Hex => format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b),
            ColorFormat::Rgb => format!("rgb({}, {}, {})", self.r, self.g, self.b),
            ColorFormat::Hsl => {
                let (h, s, l) = self.to_hsl();
                format!("hsl({h}, {s}%, {l}%)")
            }
        }
    }

    fn to_hsl(self) -> (u16, u8, u8) {
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
    fn black_hsl() {
        let c = Color::new(0, 0, 0);
        assert_eq!(c.format(ColorFormat::Hsl), "hsl(0, 0%, 0%)");
    }

    #[test]
    fn white_hsl() {
        let c = Color::new(255, 255, 255);
        assert_eq!(c.format(ColorFormat::Hsl), "hsl(0, 0%, 100%)");
    }
}
