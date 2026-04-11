use tiny_skia::Rect;

use crate::element::{Element, FreeDrawElement, LineElement, ShapeElement, TextElement};
use crate::geometry;
use crate::point::Bounds;

use super::{
    HIT_TEST_PAD, LINE_HIT_TOLERANCE, TEXT_CHAR_WIDTH_FACTOR, TEXT_LINE_HEIGHT_FACTOR,
    TEXT_MIN_CHARS,
};

impl super::Renderer {
    pub(super) fn hit_test_element(&self, el: &Element, wx: f32, wy: f32) -> bool {
        match el {
            Element::Rectangle(e) => hit_test_shape_bounds(e, wx, wy),
            Element::Ellipse(e) => hit_test_ellipse(e, wx, wy),
            Element::Diamond(e) => hit_test_diamond(e, wx, wy),
            Element::Line(e) | Element::Arrow(e) => hit_test_line(e, wx, wy),
            Element::FreeDraw(e) => hit_test_freedraw_bounds(e, wx, wy),
            Element::Text(e) => hit_test_text_bounds(e, wx, wy),
        }
    }
}

fn hit_test_shape_bounds(e: &ShapeElement, wx: f32, wy: f32) -> bool {
    let (nx, ny, nw, nh) = geometry::normalize_bounds(e.x, e.y, e.width, e.height);
    let (x, y, w, h) = (nx as f32, ny as f32, nw as f32, nh as f32);
    wx >= x - HIT_TEST_PAD
        && wx <= x + w + HIT_TEST_PAD
        && wy >= y - HIT_TEST_PAD
        && wy <= y + h + HIT_TEST_PAD
}

fn hit_test_ellipse(e: &ShapeElement, wx: f32, wy: f32) -> bool {
    let cx = e.x as f32 + e.width as f32 / 2.0;
    let cy = e.y as f32 + e.height as f32 / 2.0;
    let rx = (e.width as f32).abs() / 2.0 + HIT_TEST_PAD;
    let ry = (e.height as f32).abs() / 2.0 + HIT_TEST_PAD;
    if rx < 0.01 || ry < 0.01 {
        return false;
    }
    let dx = wx - cx;
    let dy = wy - cy;
    (dx * dx) / (rx * rx) + (dy * dy) / (ry * ry) <= 1.0
}

fn hit_test_diamond(e: &ShapeElement, wx: f32, wy: f32) -> bool {
    let (nx, ny, nw, nh) = geometry::normalize_bounds(e.x, e.y, e.width, e.height);
    let (x, y, w, h) = (nx as f32, ny as f32, nw as f32, nh as f32);
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    let dx = (wx - cx).abs();
    let dy = (wy - cy).abs();
    let hw = w / 2.0 + HIT_TEST_PAD;
    let hh = h / 2.0 + HIT_TEST_PAD;

    if hw < 0.01 || hh < 0.01 {
        return false;
    }
    dx / hw + dy / hh <= 1.0
}

fn hit_test_line(e: &LineElement, wx: f32, wy: f32) -> bool {
    let ox = e.x as f32;
    let oy = e.y as f32;
    for pair in e.points.windows(2) {
        let ax = pair[0].x as f32 + ox;
        let ay = pair[0].y as f32 + oy;
        let bx = pair[1].x as f32 + ox;
        let by = pair[1].y as f32 + oy;
        if point_to_segment_distance(wx, wy, ax, ay, bx, by) < LINE_HIT_TOLERANCE {
            return true;
        }
    }
    false
}

fn hit_test_freedraw_bounds(e: &FreeDrawElement, wx: f32, wy: f32) -> bool {
    let abs: Vec<crate::Point> = e
        .points
        .iter()
        .map(|p| crate::Point::new(p.x + e.x, p.y + e.y))
        .collect();
    let b = Bounds::from_points(&abs).unwrap_or(Bounds::new(e.x, e.y, 0.0, 0.0));
    wx >= b.x as f32 - HIT_TEST_PAD
        && wx <= (b.x + b.width) as f32 + HIT_TEST_PAD
        && wy >= b.y as f32 - HIT_TEST_PAD
        && wy <= (b.y + b.height) as f32 + HIT_TEST_PAD
}

fn hit_test_text_bounds(e: &TextElement, wx: f32, wy: f32) -> bool {
    let lines: Vec<&str> = e.text.split('\n').collect();
    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let tw =
        (max_len.max(TEXT_MIN_CHARS as usize) as f32) * e.font.size as f32 * TEXT_CHAR_WIDTH_FACTOR;
    let th = lines.len() as f32 * e.font.size as f32 * TEXT_LINE_HEIGHT_FACTOR;
    wx >= e.x as f32 - HIT_TEST_PAD
        && wx <= e.x as f32 + tw + HIT_TEST_PAD
        && wy >= e.y as f32 - HIT_TEST_PAD
        && wy <= e.y as f32 + th + HIT_TEST_PAD
}

/// Distance from point (px, py) to line segment (ax,ay)-(bx,by).
pub(super) fn point_to_segment_distance(
    px: f32,
    py: f32,
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 0.0001 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj_x = ax + t * dx;
    let proj_y = ay + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

/// Get element bounds as a tiny_skia Rect.
pub(super) fn element_bounds_f32(el: &Element) -> Option<Rect> {
    let b = el.bounds();
    Rect::from_xywh(
        b.x as f32,
        b.y as f32,
        (b.width as f32).max(0.1),
        (b.height as f32).max(0.1),
    )
}

/// Convert a Bounds to a tiny_skia Rect.
pub(super) fn to_skia_rect_from_bounds(b: &Bounds) -> Option<Rect> {
    Rect::from_xywh(
        b.x as f32,
        b.y as f32,
        (b.width as f32).max(0.1),
        (b.height as f32).max(0.1),
    )
}

/// Check if two rects intersect.
pub(super) fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    a.x() < b.x() + b.width()
        && a.x() + a.width() > b.x()
        && a.y() < b.y() + b.height()
        && a.y() + a.height() > b.y()
}
