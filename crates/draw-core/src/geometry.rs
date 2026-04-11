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
}
