use tiny_skia::*;

use crate::element::{FreeDrawElement, LineElement};

use super::CORNER_RADIUS;

pub(super) fn build_rounded_rect_path(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<Path> {
    let r = radius.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();

    if r <= 0.5 {
        let rect = Rect::from_xywh(x, y, w, h)?;
        pb.push_rect(rect);
    } else {
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        pb.quad_to(x + w, y, x + w, y + r);
        pb.line_to(x + w, y + h - r);
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        pb.line_to(x + r, y + h);
        pb.quad_to(x, y + h, x, y + h - r);
        pb.line_to(x, y + r);
        pb.quad_to(x, y, x + r, y);
        pb.close();
    }
    pb.finish()
}

pub(super) fn build_ellipse_path(cx: f32, cy: f32, rx: f32, ry: f32) -> Option<Path> {
    const KAPPA: f32 = 0.552_284_8;
    let kx = KAPPA * rx;
    let ky = KAPPA * ry;

    let mut pb = PathBuilder::new();
    pb.move_to(cx + rx, cy);
    pb.cubic_to(cx + rx, cy + ky, cx + kx, cy + ry, cx, cy + ry);
    pb.cubic_to(cx - kx, cy + ry, cx - rx, cy + ky, cx - rx, cy);
    pb.cubic_to(cx - rx, cy - ky, cx - kx, cy - ry, cx, cy - ry);
    pb.cubic_to(cx + kx, cy - ry, cx + rx, cy - ky, cx + rx, cy);
    pb.close();
    pb.finish()
}

pub(super) fn build_diamond_path(x: f32, y: f32, w: f32, h: f32) -> Option<Path> {
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;
    let r = CORNER_RADIUS.min(w / 6.0).min(h / 6.0);

    let top = (cx, y);
    let right = (x + w, cy);
    let bottom = (cx, y + h);
    let left = (x, cy);

    let dist_lt = dist_f32(left, top);
    let dist_tr = dist_f32(top, right);
    let dist_rb = dist_f32(right, bottom);
    let dist_bl = dist_f32(bottom, left);

    if dist_lt < 0.01 || dist_tr < 0.01 || dist_rb < 0.01 || dist_bl < 0.01 {
        return None;
    }

    let t_lt = (r / dist_lt).min(0.5);
    let t_tr = (r / dist_tr).min(0.5);
    let t_rb = (r / dist_rb).min(0.5);
    let t_bl = (r / dist_bl).min(0.5);

    let mut pb = PathBuilder::new();
    pb.move_to(lerp_f32(left.0, top.0, t_lt), lerp_f32(left.1, top.1, t_lt));
    pb.quad_to(
        top.0,
        top.1,
        lerp_f32(top.0, right.0, t_tr),
        lerp_f32(top.1, right.1, t_tr),
    );
    pb.quad_to(
        right.0,
        right.1,
        lerp_f32(right.0, bottom.0, t_rb),
        lerp_f32(right.1, bottom.1, t_rb),
    );
    pb.quad_to(
        bottom.0,
        bottom.1,
        lerp_f32(bottom.0, left.0, t_bl),
        lerp_f32(bottom.1, left.1, t_bl),
    );
    pb.quad_to(
        left.0,
        left.1,
        lerp_f32(left.0, top.0, t_lt),
        lerp_f32(left.1, top.1, t_lt),
    );
    pb.close();
    pb.finish()
}

pub(super) fn build_polyline_path(el: &LineElement) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let first = el.points.first()?;
    pb.move_to(first.x as f32 + el.x as f32, first.y as f32 + el.y as f32);
    for p in &el.points[1..] {
        pb.line_to(p.x as f32 + el.x as f32, p.y as f32 + el.y as f32);
    }
    pb.finish()
}

pub(super) fn build_freedraw_path(el: &FreeDrawElement) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let ox = el.x as f32;
    let oy = el.y as f32;
    let pts = &el.points;

    pb.move_to(pts[0].x as f32 + ox, pts[0].y as f32 + oy);
    for i in 1..pts.len().saturating_sub(1) {
        let xc = (pts[i].x as f32 + pts[i + 1].x as f32) / 2.0 + ox;
        let yc = (pts[i].y as f32 + pts[i + 1].y as f32) / 2.0 + oy;
        pb.quad_to(pts[i].x as f32 + ox, pts[i].y as f32 + oy, xc, yc);
    }
    let last = pts.last()?;
    pb.line_to(last.x as f32 + ox, last.y as f32 + oy);
    pb.finish()
}

pub(super) fn build_circle_path(cx: f32, cy: f32, r: f32) -> Option<Path> {
    build_ellipse_path(cx, cy, r, r)
}

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn dist_f32(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}
