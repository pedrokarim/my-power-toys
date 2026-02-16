use serde::{Deserialize, Serialize};

/// A measurement between two screen points.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Measurement {
    pub start: Point,
    pub end: Point,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Measurement {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Horizontal distance in pixels.
    pub fn width(&self) -> i32 {
        (self.end.x - self.start.x).abs()
    }

    /// Vertical distance in pixels.
    pub fn height(&self) -> i32 {
        (self.end.y - self.start.y).abs()
    }

    /// Diagonal distance in pixels.
    pub fn diagonal(&self) -> f64 {
        let dx = (self.end.x - self.start.x) as f64;
        let dy = (self.end.y - self.start.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    /// Format as human-readable string.
    pub fn display_string(&self) -> String {
        format!(
            "{}x{} px (diagonal: {:.1} px)",
            self.width(),
            self.height(),
            self.diagonal()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn horizontal_measurement() {
        let m = Measurement::new(Point { x: 0, y: 0 }, Point { x: 100, y: 0 });
        assert_eq!(m.width(), 100);
        assert_eq!(m.height(), 0);
        assert!((m.diagonal() - 100.0).abs() < 0.01);
    }

    #[test]
    fn diagonal_measurement() {
        let m = Measurement::new(Point { x: 0, y: 0 }, Point { x: 3, y: 4 });
        assert_eq!(m.width(), 3);
        assert_eq!(m.height(), 4);
        assert!((m.diagonal() - 5.0).abs() < 0.01);
    }

    #[test]
    fn display_format() {
        let m = Measurement::new(Point { x: 10, y: 20 }, Point { x: 110, y: 80 });
        let s = m.display_string();
        assert!(s.contains("100x60 px"));
    }
}
