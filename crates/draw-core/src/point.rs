use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Bounds {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for p in points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
        Some(Self::new(min_x, min_y, max_x - min_x, max_y - min_y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewState {
    pub scroll_x: f64,
    pub scroll_y: f64,
    pub zoom: f64,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(3.0, 4.0);
        assert!((a.distance_to(&b) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10.0, 10.0, 100.0, 50.0);
        assert!(bounds.contains(&Point::new(50.0, 30.0)));
        assert!(bounds.contains(&Point::new(10.0, 10.0))); // edge
        assert!(!bounds.contains(&Point::new(5.0, 30.0))); // outside left
        assert!(!bounds.contains(&Point::new(50.0, 70.0))); // outside bottom
    }

    #[test]
    fn test_bounds_intersects() {
        let a = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let b = Bounds::new(5.0, 5.0, 10.0, 10.0);
        let c = Bounds::new(20.0, 20.0, 5.0, 5.0);
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_bounds_from_points() {
        let points = vec![
            Point::new(1.0, 2.0),
            Point::new(5.0, 8.0),
            Point::new(3.0, 4.0),
        ];
        let bounds = Bounds::from_points(&points).unwrap();
        assert_eq!(bounds.x, 1.0);
        assert_eq!(bounds.y, 2.0);
        assert_eq!(bounds.width, 4.0);
        assert_eq!(bounds.height, 6.0);

        // empty
        assert!(Bounds::from_points(&[]).is_none());
    }

    #[test]
    fn test_view_state_default() {
        let vs = ViewState::default();
        assert_eq!(vs.scroll_x, 0.0);
        assert_eq!(vs.scroll_y, 0.0);
        assert_eq!(vs.zoom, 1.0);
    }
}
