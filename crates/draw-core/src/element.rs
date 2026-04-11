use serde::{Deserialize, Serialize};

use crate::point::{Bounds, Point};
use crate::style::{Arrowhead, FillStyle, FontStyle, StrokeStyle};

/// Dispatch through all Element variants, binding the inner struct to `$e`.
macro_rules! with_element {
    ($self:expr, $e:ident => $body:expr) => {
        match $self {
            Element::Rectangle($e) | Element::Ellipse($e) | Element::Diamond($e) => $body,
            Element::Line($e) | Element::Arrow($e) => $body,
            Element::FreeDraw($e) => $body,
            Element::Text($e) => $body,
        }
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    Rectangle(ShapeElement),
    Ellipse(ShapeElement),
    Diamond(ShapeElement),
    Line(LineElement),
    Arrow(LineElement),
    FreeDraw(FreeDrawElement),
    Text(TextElement),
}

impl Element {
    pub fn id(&self) -> &str {
        with_element!(self, e => &e.id)
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Rectangle(e) | Self::Ellipse(e) | Self::Diamond(e) => {
                Bounds::new(e.x, e.y, e.width, e.height)
            }
            Self::Line(e) | Self::Arrow(e) => {
                let abs: Vec<Point> = e
                    .points
                    .iter()
                    .map(|p| Point::new(p.x + e.x, p.y + e.y))
                    .collect();
                Bounds::from_points(&abs).unwrap_or(Bounds::new(e.x, e.y, 0.0, 0.0))
            }
            Self::FreeDraw(e) => {
                let abs: Vec<Point> = e
                    .points
                    .iter()
                    .map(|p| Point::new(p.x + e.x, p.y + e.y))
                    .collect();
                Bounds::from_points(&abs).unwrap_or(Bounds::new(e.x, e.y, 0.0, 0.0))
            }
            Self::Text(e) => {
                let lines: Vec<&str> = e.text.split('\n').collect();
                let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
                let width = max_len as f64 * e.font.size * 0.6;
                let height = lines.len() as f64 * e.font.size * 1.2;
                Bounds::new(e.x, e.y, width, height)
            }
        }
    }

    pub fn position(&self) -> (f64, f64) {
        with_element!(self, e => (e.x, e.y))
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        with_element!(self, e => { e.x = x; e.y = y; })
    }

    pub fn opacity(&self) -> f64 {
        with_element!(self, e => e.opacity)
    }

    pub fn is_locked(&self) -> bool {
        with_element!(self, e => e.locked)
    }

    pub fn group_id(&self) -> Option<&str> {
        with_element!(self, e => e.group_id.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub angle: f64,
    #[serde(default)]
    pub stroke: StrokeStyle,
    #[serde(default)]
    pub fill: FillStyle,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub group_id: Option<String>,
}

impl ShapeElement {
    pub fn new(id: String, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            angle: 0.0,
            stroke: StrokeStyle::default(),
            fill: FillStyle::default(),
            opacity: 1.0,
            locked: false,
            group_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub points: Vec<Point>,
    #[serde(default)]
    pub stroke: StrokeStyle,
    #[serde(default)]
    pub start_arrowhead: Option<Arrowhead>,
    #[serde(default)]
    pub end_arrowhead: Option<Arrowhead>,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub group_id: Option<String>,
}

impl LineElement {
    pub fn new(id: String, x: f64, y: f64, points: Vec<Point>) -> Self {
        Self {
            id,
            x,
            y,
            points,
            stroke: StrokeStyle::default(),
            start_arrowhead: None,
            end_arrowhead: None,
            opacity: 1.0,
            locked: false,
            group_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FreeDrawElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub points: Vec<Point>,
    #[serde(default)]
    pub stroke: StrokeStyle,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub group_id: Option<String>,
}

impl FreeDrawElement {
    pub fn new(id: String, x: f64, y: f64, points: Vec<Point>) -> Self {
        Self {
            id,
            x,
            y,
            points,
            stroke: StrokeStyle::default(),
            opacity: 1.0,
            locked: false,
            group_id: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextElement {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub text: String,
    #[serde(default)]
    pub font: FontStyle,
    #[serde(default)]
    pub stroke: StrokeStyle,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub angle: f64,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub group_id: Option<String>,
}

impl TextElement {
    pub fn new(id: String, x: f64, y: f64, text: String) -> Self {
        Self {
            id,
            x,
            y,
            text,
            font: FontStyle::default(),
            stroke: StrokeStyle::default(),
            opacity: 1.0,
            angle: 0.0,
            locked: false,
            group_id: None,
        }
    }
}

fn default_opacity() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_id() {
        let rect = Element::Rectangle(ShapeElement::new("r1".to_string(), 0.0, 0.0, 10.0, 10.0));
        assert_eq!(rect.id(), "r1");

        let line = Element::Line(LineElement::new(
            "l1".to_string(),
            0.0,
            0.0,
            vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)],
        ));
        assert_eq!(line.id(), "l1");

        let text = Element::Text(TextElement::new(
            "t1".to_string(),
            0.0,
            0.0,
            "hello".to_string(),
        ));
        assert_eq!(text.id(), "t1");

        let fd = Element::FreeDraw(FreeDrawElement::new(
            "fd1".to_string(),
            0.0,
            0.0,
            vec![Point::new(0.0, 0.0)],
        ));
        assert_eq!(fd.id(), "fd1");
    }

    #[test]
    fn test_element_bounds_rectangle() {
        let rect = Element::Rectangle(ShapeElement::new("r1".to_string(), 10.0, 20.0, 100.0, 50.0));
        let b = rect.bounds();
        assert_eq!(b.x, 10.0);
        assert_eq!(b.y, 20.0);
        assert_eq!(b.width, 100.0);
        assert_eq!(b.height, 50.0);
    }

    #[test]
    fn test_element_bounds_ellipse() {
        let ellipse = Element::Ellipse(ShapeElement::new("e1".to_string(), 5.0, 10.0, 80.0, 60.0));
        let b = ellipse.bounds();
        assert_eq!(b.x, 5.0);
        assert_eq!(b.y, 10.0);
        assert_eq!(b.width, 80.0);
        assert_eq!(b.height, 60.0);
    }

    #[test]
    fn test_element_bounds_line() {
        let line = Element::Line(LineElement::new(
            "l1".to_string(),
            10.0,
            20.0,
            vec![Point::new(0.0, 0.0), Point::new(50.0, 30.0)],
        ));
        let b = line.bounds();
        // Absolute points: (10,20) and (60,50)
        assert_eq!(b.x, 10.0);
        assert_eq!(b.y, 20.0);
        assert_eq!(b.width, 50.0);
        assert_eq!(b.height, 30.0);
    }

    #[test]
    fn test_element_bounds_freedraw() {
        let fd = Element::FreeDraw(FreeDrawElement::new(
            "fd1".to_string(),
            5.0,
            5.0,
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 20.0),
                Point::new(-5.0, 10.0),
            ],
        ));
        let b = fd.bounds();
        // Absolute points: (5,5), (15,25), (0,15)
        assert_eq!(b.x, 0.0);
        assert_eq!(b.y, 5.0);
        assert_eq!(b.width, 15.0);
        assert_eq!(b.height, 20.0);
    }

    #[test]
    fn test_element_bounds_text_multiline() {
        let text = Element::Text(TextElement::new(
            "t1".to_string(),
            0.0,
            0.0,
            "hello\nworld!!".to_string(),
        ));
        let b = text.bounds();
        // "world!!" is 7 chars (longest line), font size default 16
        // width = 7 * 16 * 0.6 = 67.2
        // height = 2 * 16 * 1.2 = 38.4
        assert!(b.width > 0.0);
        assert!(b.height > b.width * 0.3); // 2 lines should be taller than single
        // Verify it accounts for 2 lines
        let single = Element::Text(TextElement::new(
            "t2".to_string(),
            0.0,
            0.0,
            "hello".to_string(),
        ));
        let sb = single.bounds();
        assert!(b.height > sb.height);
    }
}
