use tiny_skia::*;

use crate::element::{Element, FreeDrawElement, LineElement, ShapeElement, TextElement};
use crate::geometry;
use crate::style::{FillStyle, FillType};

use super::path::*;
use super::{
    CORNER_RADIUS, GRID_ALPHA, GRID_B, GRID_G, GRID_MIN_SCREEN_PX, GRID_R, GRID_SIZE,
    HACHURE_ALPHA, HACHURE_LINE_WIDTH, parse_color, stroke_from_style,
};

impl super::Renderer {
    pub(super) fn draw_element(&self, pixmap: &mut Pixmap, el: &Element, transform: &Transform) {
        let opacity = el.opacity() as f32;
        match el {
            Element::Rectangle(e) => self.draw_rectangle(pixmap, e, transform, opacity),
            Element::Ellipse(e) => self.draw_ellipse(pixmap, e, transform, opacity),
            Element::Diamond(e) => self.draw_diamond(pixmap, e, transform, opacity),
            Element::Line(e) => self.draw_line(pixmap, e, transform, opacity),
            Element::Arrow(e) => self.draw_arrow(pixmap, e, transform, opacity),
            Element::FreeDraw(e) => self.draw_freedraw(pixmap, e, transform, opacity),
            Element::Text(e) => self.draw_text(pixmap, e, transform, opacity),
        }
    }

    fn draw_rectangle(
        &self,
        pixmap: &mut Pixmap,
        el: &ShapeElement,
        transform: &Transform,
        opacity: f32,
    ) {
        let (nx, ny, nw, nh) = geometry::normalize_bounds(el.x, el.y, el.width, el.height);
        let (x, y, w, h) = (nx as f32, ny as f32, nw as f32, nh as f32);
        if w < 0.5 && h < 0.5 {
            return;
        }

        let radius = CORNER_RADIUS.min(w / 3.0).min(h / 3.0);
        if let Some(path) = build_rounded_rect_path(x, y, w, h, radius) {
            self.fill_shape(pixmap, &path, &el.fill, x, y, w, h, transform, opacity);
            let (paint, stroke) = stroke_from_style(&el.stroke, opacity);
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
        }
    }

    fn draw_ellipse(
        &self,
        pixmap: &mut Pixmap,
        el: &ShapeElement,
        transform: &Transform,
        opacity: f32,
    ) {
        let rx = (el.width).abs() as f32 / 2.0;
        let ry = (el.height).abs() as f32 / 2.0;
        if rx < 0.5 && ry < 0.5 {
            return;
        }

        let cx = el.x as f32 + el.width as f32 / 2.0;
        let cy = el.y as f32 + el.height as f32 / 2.0;
        let safe_rx = rx.max(0.1);
        let safe_ry = ry.max(0.1);

        if let Some(path) = build_ellipse_path(cx, cy, safe_rx, safe_ry) {
            let (nx, ny, nw, nh) = geometry::normalize_bounds(el.x, el.y, el.width, el.height);
            let (bx, by, w, h) = (nx as f32, ny as f32, nw as f32, nh as f32);

            self.fill_shape(pixmap, &path, &el.fill, bx, by, w, h, transform, opacity);
            let (paint, stroke) = stroke_from_style(&el.stroke, opacity);
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
        }
    }

    fn draw_diamond(
        &self,
        pixmap: &mut Pixmap,
        el: &ShapeElement,
        transform: &Transform,
        opacity: f32,
    ) {
        let (nx, ny, nw, nh) = geometry::normalize_bounds(el.x, el.y, el.width, el.height);
        let (x, y, w, h) = (nx as f32, ny as f32, nw as f32, nh as f32);
        if w < 0.5 && h < 0.5 {
            return;
        }

        if let Some(path) = build_diamond_path(x, y, w, h) {
            self.fill_shape(pixmap, &path, &el.fill, x, y, w, h, transform, opacity);
            let (paint, stroke) = stroke_from_style(&el.stroke, opacity);
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
        }
    }

    fn draw_line(
        &self,
        pixmap: &mut Pixmap,
        el: &LineElement,
        transform: &Transform,
        opacity: f32,
    ) {
        if el.points.len() < 2 {
            return;
        }
        if let Some(path) = build_polyline_path(el) {
            let (paint, stroke) = stroke_from_style(&el.stroke, opacity);
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
        }
    }

    fn draw_arrow(
        &self,
        pixmap: &mut Pixmap,
        el: &LineElement,
        transform: &Transform,
        opacity: f32,
    ) {
        self.draw_line(pixmap, el, transform, opacity);
        if el.points.len() < 2 {
            return;
        }

        let color = parse_color(&el.stroke.color, opacity);
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        let arrowhead_stroke = Stroke {
            width: (el.stroke.width as f32) * 0.5,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };

        let last = el.points.last().unwrap();
        let prev = &el.points[el.points.len() - 2];
        let end_ah = geometry::compute_arrowhead(
            last.x + el.x,
            last.y + el.y,
            prev.x + el.x,
            prev.y + el.y,
            geometry::ARROWHEAD_LENGTH,
            geometry::ARROWHEAD_ANGLE,
        );
        draw_arrowhead_path(pixmap, &end_ah, &paint, &arrowhead_stroke, transform);

        if el.start_arrowhead.is_some() {
            let first = &el.points[0];
            let next = &el.points[1];
            let start_ah = geometry::compute_arrowhead(
                first.x + el.x,
                first.y + el.y,
                next.x + el.x,
                next.y + el.y,
                geometry::ARROWHEAD_LENGTH,
                geometry::ARROWHEAD_ANGLE,
            );
            draw_arrowhead_path(pixmap, &start_ah, &paint, &arrowhead_stroke, transform);
        }
    }

    fn draw_freedraw(
        &self,
        pixmap: &mut Pixmap,
        el: &FreeDrawElement,
        transform: &Transform,
        opacity: f32,
    ) {
        if el.points.len() < 2 {
            return;
        }
        if let Some(path) = build_freedraw_path(el) {
            let (paint, stroke) = stroke_from_style(&el.stroke, opacity);
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
        }
    }

    fn draw_text(
        &self,
        _pixmap: &mut Pixmap,
        _el: &TextElement,
        _transform: &Transform,
        _opacity: f32,
    ) {
        // Text is rendered by the browser via canvas overlays (renderTextOverlays).
        // The WASM renderer intentionally draws nothing here.
    }

    // ── Fill patterns ───────────────────────────────────────────────

    #[allow(clippy::too_many_arguments)]
    fn fill_shape(
        &self,
        pixmap: &mut Pixmap,
        clip_path: &Path,
        fill: &FillStyle,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        transform: &Transform,
        opacity: f32,
    ) {
        match fill.style {
            FillType::None => {}
            FillType::Solid => {
                let color = parse_color(&fill.color, opacity);
                let mut paint = Paint::default();
                paint.set_color(color);
                paint.anti_alias = true;
                pixmap.fill_path(clip_path, &paint, FillRule::Winding, *transform, None);
            }
            FillType::Hachure => {
                self.draw_hachure_fill(
                    pixmap,
                    clip_path,
                    &fill.color,
                    fill.gap as f32,
                    fill.angle as f32,
                    x,
                    y,
                    w,
                    h,
                    transform,
                    opacity,
                );
            }
            FillType::CrossHatch => {
                self.draw_hachure_fill(
                    pixmap,
                    clip_path,
                    &fill.color,
                    fill.gap as f32,
                    fill.angle as f32,
                    x,
                    y,
                    w,
                    h,
                    transform,
                    opacity,
                );
                self.draw_hachure_fill(
                    pixmap,
                    clip_path,
                    &fill.color,
                    fill.gap as f32,
                    fill.angle as f32 + std::f32::consts::FRAC_PI_2,
                    x,
                    y,
                    w,
                    h,
                    transform,
                    opacity,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_hachure_fill(
        &self,
        pixmap: &mut Pixmap,
        clip_path: &Path,
        color: &str,
        gap: f32,
        angle: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        transform: &Transform,
        opacity: f32,
    ) {
        let mut clip_mask = match Mask::new(pixmap.width(), pixmap.height()) {
            Some(m) => m,
            None => return,
        };
        clip_mask.fill_path(clip_path, FillRule::Winding, true, *transform);

        let parsed_color = parse_color(color, opacity * HACHURE_ALPHA);
        let mut paint = Paint::default();
        paint.set_color(parsed_color);
        paint.anti_alias = true;

        let stroke = Stroke {
            width: HACHURE_LINE_WIDTH,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };

        let cx = (x + w / 2.0) as f64;
        let cy = (y + h / 2.0) as f64;
        for line in
            geometry::generate_hachure_lines(cx, cy, w as f64, h as f64, gap as f64, angle as f64)
        {
            let mut pb = PathBuilder::new();
            pb.move_to(line.x1 as f32, line.y1 as f32);
            pb.line_to(line.x2 as f32, line.y2 as f32);
            if let Some(line_path) = pb.finish() {
                pixmap.stroke_path(&line_path, &paint, &stroke, *transform, Some(&clip_mask));
            }
        }
    }

    // ── Grid ────────────────────────────────────────────────────────

    pub(super) fn draw_grid(&self, pixmap: &mut Pixmap, viewport: &crate::point::ViewState) {
        let zoom = viewport.zoom as f32 * self.config.pixel_ratio;
        let gs = GRID_SIZE * zoom;
        if gs < GRID_MIN_SCREEN_PX {
            return;
        }

        let w = pixmap.width() as f32;
        let h = pixmap.height() as f32;
        let off_x = (viewport.scroll_x as f32 * self.config.pixel_ratio) % gs;
        let off_y = (viewport.scroll_y as f32 * self.config.pixel_ratio) % gs;

        let color = Color::from_rgba(
            GRID_R as f32 / 255.0,
            GRID_G as f32 / 255.0,
            GRID_B as f32 / 255.0,
            GRID_ALPHA,
        )
        .unwrap_or(Color::TRANSPARENT);
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = false;

        let stroke = Stroke {
            width: 1.0,
            ..Stroke::default()
        };
        let identity = Transform::identity();

        let mut x = off_x;
        while x < w {
            let mut pb = PathBuilder::new();
            pb.move_to(x, 0.0);
            pb.line_to(x, h);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, identity, None);
            }
            x += gs;
        }

        let mut y = off_y;
        while y < h {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, y);
            pb.line_to(w, y);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, identity, None);
            }
            y += gs;
        }
    }
}

fn draw_arrowhead_path(
    pixmap: &mut Pixmap,
    ah: &geometry::ArrowheadPoints,
    paint: &Paint,
    stroke: &Stroke,
    transform: &Transform,
) {
    let mut pb = PathBuilder::new();
    pb.move_to(ah.tip_x as f32, ah.tip_y as f32);
    pb.line_to(ah.left_x as f32, ah.left_y as f32);
    pb.line_to(ah.right_x as f32, ah.right_y as f32);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.fill_path(&path, paint, FillRule::Winding, *transform, None);
        pixmap.stroke_path(&path, paint, stroke, *transform, None);
    }
}
