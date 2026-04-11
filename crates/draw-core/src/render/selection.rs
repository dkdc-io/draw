use tiny_skia::*;

use crate::element::Element;
use crate::point::{Bounds, ViewState};

use super::path::build_circle_path;
use super::{
    ACCENT_B, ACCENT_G, ACCENT_R, HANDLE_FILL_B, HANDLE_FILL_G, HANDLE_FILL_R, HANDLE_RADIUS,
    SELECTION_DASH_LEN, SELECTION_FILL_ALPHA, SELECTION_PAD, viewport_transform,
};

impl super::Renderer {
    pub(super) fn draw_selection_box(
        &self,
        pixmap: &mut Pixmap,
        el: &Element,
        viewport: &ViewState,
    ) {
        let b = el.bounds();
        let scale = viewport.zoom as f32 * self.config.pixel_ratio;
        let vt = viewport_transform(viewport, self.config.pixel_ratio);

        let pad = SELECTION_PAD;
        let sx = b.x as f32 - pad;
        let sy = b.y as f32 - pad;
        let sw = b.width as f32 + pad * 2.0;
        let sh = b.height as f32 + pad * 2.0;

        if let Some(rect) = Rect::from_xywh(sx, sy, sw.max(1.0), sh.max(1.0)) {
            // Selection fill
            let fill_color = Color::from_rgba(
                ACCENT_R as f32 / 255.0,
                ACCENT_G as f32 / 255.0,
                ACCENT_B as f32 / 255.0,
                SELECTION_FILL_ALPHA,
            )
            .unwrap_or(Color::TRANSPARENT);
            let mut paint = Paint::default();
            paint.set_color(fill_color);
            pixmap.fill_rect(rect, &paint, vt, None);

            // Dashed border
            let accent = Color::from_rgba8(ACCENT_R, ACCENT_G, ACCENT_B, 255);
            let mut border_paint = Paint::default();
            border_paint.set_color(accent);
            border_paint.anti_alias = true;
            let dash_len = SELECTION_DASH_LEN / scale;
            let stroke = Stroke {
                width: 1.5 / scale,
                dash: StrokeDash::new(vec![dash_len, dash_len], 0.0),
                ..Stroke::default()
            };
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &border_paint, &stroke, vt, None);
            }

            // Resize handles at corners
            let handle_fill = {
                let c = Color::from_rgba8(HANDLE_FILL_R, HANDLE_FILL_G, HANDLE_FILL_B, 255);
                let mut p = Paint::default();
                p.set_color(c);
                p.anti_alias = true;
                p
            };
            let mut handle_stroke_paint = Paint::default();
            handle_stroke_paint.set_color(accent);
            handle_stroke_paint.anti_alias = true;
            let handle_stroke = Stroke {
                width: 1.5 / scale,
                ..Stroke::default()
            };
            let hs = HANDLE_RADIUS / scale;

            let corners = [(sx, sy), (sx + sw, sy), (sx, sy + sh), (sx + sw, sy + sh)];
            for (hx, hy) in &corners {
                if let Some(handle_path) = build_circle_path(*hx, *hy, hs) {
                    pixmap.fill_path(&handle_path, &handle_fill, FillRule::Winding, vt, None);
                    pixmap.stroke_path(
                        &handle_path,
                        &handle_stroke_paint,
                        &handle_stroke,
                        vt,
                        None,
                    );
                }
            }
        }
    }

    pub(super) fn draw_rubber_band(&self, pixmap: &mut Pixmap, sb: &Bounds) {
        let pr = self.config.pixel_ratio;
        let x = sb.x as f32 * pr;
        let y = sb.y as f32 * pr;
        let w = (sb.width as f32 * pr).abs().max(1.0);
        let h = (sb.height as f32 * pr).abs().max(1.0);

        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            let identity = Transform::identity();

            let fill_color = Color::from_rgba(
                ACCENT_R as f32 / 255.0,
                ACCENT_G as f32 / 255.0,
                ACCENT_B as f32 / 255.0,
                SELECTION_FILL_ALPHA,
            )
            .unwrap_or(Color::TRANSPARENT);
            let mut fill_paint = Paint::default();
            fill_paint.set_color(fill_color);
            pixmap.fill_rect(rect, &fill_paint, identity, None);

            let accent = Color::from_rgba8(ACCENT_R, ACCENT_G, ACCENT_B, 255);
            let mut stroke_paint = Paint::default();
            stroke_paint.set_color(accent);
            stroke_paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.0 * pr,
                dash: StrokeDash::new(vec![SELECTION_DASH_LEN * pr, SELECTION_DASH_LEN * pr], 0.0),
                ..Stroke::default()
            };
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, identity, None);
            }
        }
    }
}
