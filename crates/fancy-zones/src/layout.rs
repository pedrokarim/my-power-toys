use serde::{Deserialize, Serialize};

/// A zone layout definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub name: String,
    pub zones: Vec<Zone>,
}

/// A zone is a rectangular region on screen, defined as percentages (0.0 - 1.0).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Zone {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Zone {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Convert to absolute pixel coordinates for a given screen size.
    pub fn to_pixels(&self, screen_w: u32, screen_h: u32) -> (i32, i32, u32, u32) {
        let x = (self.x * screen_w as f32) as i32;
        let y = (self.y * screen_h as f32) as i32;
        let w = (self.width * screen_w as f32) as u32;
        let h = (self.height * screen_h as f32) as u32;
        (x, y, w, h)
    }

    /// Convert to pixel coordinates with gap applied (half-gap inset on each side).
    pub fn to_pixels_with_gap(
        &self,
        screen_w: u32,
        screen_h: u32,
        gap: u32,
    ) -> (i32, i32, u32, u32) {
        let (x, y, w, h) = self.to_pixels(screen_w, screen_h);
        let half = (gap / 2) as i32;
        (
            x + half,
            y + half,
            (w as i32 - gap as i32).max(100) as u32,
            (h as i32 - gap as i32).max(100) as u32,
        )
    }

    /// Check if a point (in percentages) is inside this zone.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

impl Layout {
    /// Create a layout with N equal columns.
    pub fn default_columns(n: usize) -> Self {
        let w = 1.0 / n as f32;
        let zones = (0..n)
            .map(|i| Zone::new(i as f32 * w, 0.0, w, 1.0))
            .collect();
        Self {
            name: format!("{n} Columns"),
            zones,
        }
    }

    /// Create a layout with N equal rows.
    pub fn default_rows(n: usize) -> Self {
        let h = 1.0 / n as f32;
        let zones = (0..n)
            .map(|i| Zone::new(0.0, i as f32 * h, 1.0, h))
            .collect();
        Self {
            name: format!("{n} Rows"),
            zones,
        }
    }

    /// Create a "main + side" layout (like i3/sway master-stack).
    pub fn main_plus_stack() -> Self {
        Self {
            name: "Main + Stack".to_string(),
            zones: vec![
                Zone::new(0.0, 0.0, 0.6, 1.0), // Main (60%)
                Zone::new(0.6, 0.0, 0.4, 0.5), // Stack top
                Zone::new(0.6, 0.5, 0.4, 0.5), // Stack bottom
            ],
        }
    }

    /// Create a grid layout (NxM).
    pub fn grid(cols: usize, rows: usize) -> Self {
        let w = 1.0 / cols as f32;
        let h = 1.0 / rows as f32;
        let zones = (0..rows)
            .flat_map(|r| (0..cols).map(move |c| Zone::new(c as f32 * w, r as f32 * h, w, h)))
            .collect();
        Self {
            name: format!("{cols}x{rows} Grid"),
            zones,
        }
    }

    /// Find which zone a point falls into.
    pub fn zone_at(&self, px: f32, py: f32) -> Option<usize> {
        self.zones.iter().position(|z| z.contains(px, py))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_columns() {
        let layout = Layout::default_columns(3);
        assert_eq!(layout.zones.len(), 3);
        let z = &layout.zones[0];
        assert!((z.x - 0.0).abs() < 0.001);
        assert!((z.width - 1.0 / 3.0).abs() < 0.001);
    }

    #[test]
    fn zone_pixel_conversion() {
        let z = Zone::new(0.5, 0.0, 0.5, 1.0);
        let (x, y, w, h) = z.to_pixels(1920, 1080);
        assert_eq!(x, 960);
        assert_eq!(y, 0);
        assert_eq!(w, 960);
        assert_eq!(h, 1080);
    }

    #[test]
    fn zone_contains_point() {
        let z = Zone::new(0.0, 0.0, 0.5, 1.0);
        assert!(z.contains(0.25, 0.5));
        assert!(!z.contains(0.75, 0.5));
    }

    #[test]
    fn zone_at_lookup() {
        let layout = Layout::default_columns(2);
        assert_eq!(layout.zone_at(0.25, 0.5), Some(0));
        assert_eq!(layout.zone_at(0.75, 0.5), Some(1));
    }

    #[test]
    fn main_plus_stack_layout() {
        let layout = Layout::main_plus_stack();
        assert_eq!(layout.zones.len(), 3);
        assert!(layout.zone_at(0.3, 0.5) == Some(0)); // main area
    }

    #[test]
    fn grid_layout() {
        let layout = Layout::grid(2, 2);
        assert_eq!(layout.zones.len(), 4);
        assert_eq!(layout.zone_at(0.25, 0.25), Some(0));
        assert_eq!(layout.zone_at(0.75, 0.75), Some(3));
    }

    #[test]
    fn layout_serializable() {
        let layout = Layout::main_plus_stack();
        let json = serde_json::to_string(&layout).unwrap();
        let parsed: Layout = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.zones.len(), 3);
    }
}
