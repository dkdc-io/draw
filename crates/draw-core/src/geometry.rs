//! Shared geometry helpers used by both the pixel renderer and SVG export.

/// Arrowhead geometry constants.
pub const ARROWHEAD_LENGTH: f64 = 14.0;
pub const ARROWHEAD_ANGLE: f64 = 0.45;

/// Hachure fill line width.
pub const HACHURE_LINE_WIDTH: f64 = 1.5;

/// Normalize a bounding box that may have negative width/height.
/// Returns `(x, y, abs_width, abs_height)` with the top-left corner.
pub fn normalize_bounds(x: f64, y: f64, w: f64, h: f64) -> (f64, f64, f64, f64) {
    let nx = if w < 0.0 { x + w } else { x };
    let ny = if h < 0.0 { y + h } else { y };
    (nx, ny, w.abs(), h.abs())
}

/// The three vertices of an arrowhead triangle.
pub struct ArrowheadPoints {
    pub tip_x: f64,
    pub tip_y: f64,
    pub left_x: f64,
    pub left_y: f64,
    pub right_x: f64,
    pub right_y: f64,
}

/// Compute arrowhead triangle vertices.
///
/// `tip` is the point of the arrow, `from` is the point it points away from.
/// `length` and `spread` control the size and opening angle.
pub fn compute_arrowhead(
    tip_x: f64,
    tip_y: f64,
    from_x: f64,
    from_y: f64,
    length: f64,
    spread: f64,
) -> ArrowheadPoints {
    let angle = (tip_y - from_y).atan2(tip_x - from_x);
    ArrowheadPoints {
        tip_x,
        tip_y,
        left_x: tip_x - length * (angle - spread).cos(),
        left_y: tip_y - length * (angle - spread).sin(),
        right_x: tip_x - length * (angle + spread).cos(),
        right_y: tip_y - length * (angle + spread).sin(),
    }
}

/// A single hachure line segment.
pub struct HachureLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Generate parallel hachure lines rotated around the center of a bounding box.
///
/// Lines span from `-diag` to `+diag` so they fully cover the shape before
/// clipping. `gap` is the spacing between lines and `angle` is the rotation
/// in radians.
pub fn generate_hachure_lines(
    cx: f64,
    cy: f64,
    width: f64,
    height: f64,
    gap: f64,
    angle: f64,
) -> Vec<HachureLine> {
    let diag = (width * width + height * height).sqrt();
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let mut lines = Vec::new();
    let mut d = -diag;
    while d < diag {
        lines.push(HachureLine {
            x1: cx + d * cos_a - (-diag) * sin_a,
            y1: cy + d * sin_a + (-diag) * cos_a,
            x2: cx + d * cos_a - diag * sin_a,
            y2: cy + d * sin_a + diag * cos_a,
        });
        d += gap;
    }
    lines
}

use crate::element::Element;

/// Connection point on a shape where arrows can snap.
#[derive(Debug, Clone)]
pub struct ConnectionPoint {
    pub x: f64,
    pub y: f64,
}

/// Compute the 8 connection points for a shape element.
/// Returns 4 edge midpoints + 4 corners (or shape-specific points).
/// Returns empty vec for non-shape elements (Line, Arrow, FreeDraw, Text).
pub fn connection_points(el: &Element) -> Vec<ConnectionPoint> {
    match el {
        Element::Rectangle(e) => {
            let (x, y, w, h) = normalize_bounds(e.x, e.y, e.width, e.height);
            rectangle_connection_points(x, y, w, h)
        }
        Element::Ellipse(e) => {
            let (x, y, w, h) = normalize_bounds(e.x, e.y, e.width, e.height);
            ellipse_connection_points(x, y, w, h)
        }
        Element::Diamond(e) => {
            let (x, y, w, h) = normalize_bounds(e.x, e.y, e.width, e.height);
            diamond_connection_points(x, y, w, h)
        }
        _ => Vec::new(),
    }
}

fn rectangle_connection_points(x: f64, y: f64, w: f64, h: f64) -> Vec<ConnectionPoint> {
    vec![
        // Edge midpoints
        ConnectionPoint { x: x + w / 2.0, y }, // top center
        ConnectionPoint {
            x: x + w,
            y: y + h / 2.0,
        }, // right center
        ConnectionPoint {
            x: x + w / 2.0,
            y: y + h,
        }, // bottom center
        ConnectionPoint { x, y: y + h / 2.0 }, // left center
        // Corners
        ConnectionPoint { x, y },               // top-left
        ConnectionPoint { x: x + w, y },        // top-right
        ConnectionPoint { x: x + w, y: y + h }, // bottom-right
        ConnectionPoint { x, y: y + h },        // bottom-left
    ]
}

fn ellipse_connection_points(x: f64, y: f64, w: f64, h: f64) -> Vec<ConnectionPoint> {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let rx = w / 2.0;
    let ry = h / 2.0;
    let cos45 = std::f64::consts::FRAC_PI_4.cos();
    let sin45 = std::f64::consts::FRAC_PI_4.sin();
    vec![
        // Cardinal points (edge midpoints of bounding box on ellipse perimeter)
        ConnectionPoint { x: cx, y: cy - ry }, // top
        ConnectionPoint { x: cx + rx, y: cy }, // right
        ConnectionPoint { x: cx, y: cy + ry }, // bottom
        ConnectionPoint { x: cx - rx, y: cy }, // left
        // 45-degree points on ellipse perimeter
        ConnectionPoint {
            x: cx + rx * cos45,
            y: cy - ry * sin45,
        }, // top-right
        ConnectionPoint {
            x: cx + rx * cos45,
            y: cy + ry * sin45,
        }, // bottom-right
        ConnectionPoint {
            x: cx - rx * cos45,
            y: cy + ry * sin45,
        }, // bottom-left
        ConnectionPoint {
            x: cx - rx * cos45,
            y: cy - ry * sin45,
        }, // top-left
    ]
}

fn diamond_connection_points(x: f64, y: f64, w: f64, h: f64) -> Vec<ConnectionPoint> {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    vec![
        // Vertices (the 4 tips of the diamond)
        ConnectionPoint { x: cx, y },        // top vertex
        ConnectionPoint { x: x + w, y: cy }, // right vertex
        ConnectionPoint { x: cx, y: y + h }, // bottom vertex
        ConnectionPoint { x, y: cy },        // left vertex
        // Edge midpoints (between adjacent vertices)
        ConnectionPoint {
            x: cx + w / 4.0,
            y: y + h / 4.0,
        }, // top-right edge mid
        ConnectionPoint {
            x: cx + w / 4.0,
            y: cy + h / 4.0,
        }, // bottom-right edge mid
        ConnectionPoint {
            x: cx - w / 4.0,
            y: cy + h / 4.0,
        }, // bottom-left edge mid
        ConnectionPoint {
            x: cx - w / 4.0,
            y: y + h / 4.0,
        }, // top-left edge mid
    ]
}

/// Find the nearest connection point within `threshold` world-coordinate distance.
/// `wx, wy` are the world coordinates to snap to.
/// `exclude_id` is the element to skip (the arrow being drawn).
/// Returns `(element_id, snap_x, snap_y)` or None.
pub fn find_nearest_snap_point(
    elements: &[Element],
    wx: f64,
    wy: f64,
    threshold: f64,
    exclude_id: &str,
) -> Option<(String, f64, f64)> {
    let mut best: Option<(String, f64, f64, f64)> = None; // (id, x, y, dist)

    for el in elements {
        if el.id() == exclude_id {
            continue;
        }
        let pts = connection_points(el);
        for cp in &pts {
            let dist = ((cp.x - wx).powi(2) + (cp.y - wy).powi(2)).sqrt();
            if dist < threshold && best.as_ref().is_none_or(|b| dist < b.3) {
                best = Some((el.id().to_string(), cp.x, cp.y, dist));
            }
        }
    }

    best.map(|(id, x, y, _)| (id, x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_positive_bounds() {
        let (x, y, w, h) = normalize_bounds(10.0, 20.0, 100.0, 50.0);
        assert_eq!((x, y, w, h), (10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn normalize_negative_width() {
        let (x, y, w, h) = normalize_bounds(110.0, 20.0, -100.0, 50.0);
        assert_eq!((x, y, w, h), (10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn normalize_negative_both() {
        let (x, y, w, h) = normalize_bounds(110.0, 70.0, -100.0, -50.0);
        assert_eq!((x, y, w, h), (10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn arrowhead_horizontal_right() {
        let pts = compute_arrowhead(100.0, 0.0, 0.0, 0.0, 14.0, 0.45);
        assert!((pts.tip_x - 100.0).abs() < 1e-10);
        assert!((pts.tip_y - 0.0).abs() < 1e-10);
        // Left/right should be symmetric about the axis
        assert!((pts.left_y + pts.right_y).abs() < 1e-10);
    }

    #[test]
    fn hachure_lines_count() {
        let lines = generate_hachure_lines(50.0, 50.0, 100.0, 100.0, 10.0, 0.0);
        assert!(!lines.is_empty());
        // With diag ~141.4 and gap 10, expect ~28 lines
        assert!(lines.len() > 20);
        assert!(lines.len() < 40);
    }

    #[test]
    fn hachure_lines_empty_for_huge_gap() {
        let lines = generate_hachure_lines(50.0, 50.0, 10.0, 10.0, 1000.0, 0.0);
        // diag ~14.1, gap 1000 — only one or zero lines
        assert!(lines.len() <= 1);
    }

    #[test]
    fn rectangle_connection_points_count() {
        use crate::element::{Element, ShapeElement};
        let el = Element::Rectangle(ShapeElement::new("r1".into(), 0.0, 0.0, 100.0, 50.0));
        let pts = connection_points(&el);
        assert_eq!(pts.len(), 8);
        // Top center
        assert!((pts[0].x - 50.0).abs() < 1e-10);
        assert!((pts[0].y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn ellipse_connection_points_count() {
        use crate::element::{Element, ShapeElement};
        let el = Element::Ellipse(ShapeElement::new("e1".into(), 0.0, 0.0, 100.0, 60.0));
        let pts = connection_points(&el);
        assert_eq!(pts.len(), 8);
        // Top point should be at center-x, top of ellipse
        assert!((pts[0].x - 50.0).abs() < 1e-10);
        assert!((pts[0].y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn diamond_connection_points_count() {
        use crate::element::{Element, ShapeElement};
        let el = Element::Diamond(ShapeElement::new("d1".into(), 0.0, 0.0, 100.0, 80.0));
        let pts = connection_points(&el);
        assert_eq!(pts.len(), 8);
        // Top vertex
        assert!((pts[0].x - 50.0).abs() < 1e-10);
        assert!((pts[0].y - 0.0).abs() < 1e-10);
    }

    #[test]
    fn find_snap_point_within_threshold() {
        use crate::element::{Element, ShapeElement};
        let elements = vec![Element::Rectangle(ShapeElement::new(
            "r1".into(),
            100.0,
            100.0,
            80.0,
            60.0,
        ))];
        // Top-center of rectangle is at (140, 100)
        let result = find_nearest_snap_point(&elements, 142.0, 102.0, 15.0, "");
        assert!(result.is_some());
        let (id, sx, sy) = result.unwrap();
        assert_eq!(id, "r1");
        assert!((sx - 140.0).abs() < 1e-10);
        assert!((sy - 100.0).abs() < 1e-10);
    }

    #[test]
    fn find_snap_point_outside_threshold() {
        use crate::element::{Element, ShapeElement};
        let elements = vec![Element::Rectangle(ShapeElement::new(
            "r1".into(),
            100.0,
            100.0,
            80.0,
            60.0,
        ))];
        // Far from any connection point
        let result = find_nearest_snap_point(&elements, 0.0, 0.0, 15.0, "");
        assert!(result.is_none());
    }

    #[test]
    fn find_snap_excludes_self() {
        use crate::element::{Element, ShapeElement};
        let elements = vec![Element::Rectangle(ShapeElement::new(
            "r1".into(),
            100.0,
            100.0,
            80.0,
            60.0,
        ))];
        // Within threshold but excluded
        let result = find_nearest_snap_point(&elements, 140.0, 100.0, 15.0, "r1");
        assert!(result.is_none());
    }
}
