//! Unified tiny-skia renderer for both native and WASM targets.
//!
//! This module implements the full rendering pipeline: viewport transform,
//! element drawing (shapes, lines, text placeholders), fill patterns (solid,
//! hachure, crosshatch), selection visuals, and hit testing.
//!
//! # Text rendering
//! TODO: Text is currently rendered as a colored placeholder rectangle with
//! approximate bounds. Full glyph rendering requires cosmic-text integration,
//! which will be added in a follow-up.

use tiny_skia::*;

use crate::Document;
use crate::element::{Element, FreeDrawElement, LineElement, ShapeElement, TextElement};
use crate::point::{Bounds, ViewState};
use crate::style::{FillStyle, FillType, StrokeStyle};

// ── Theme constants (matches frontend/theme.js) ────────────────────────

/// Canvas background color (#0a0f1a)
const BG_R: u8 = 10;
const BG_G: u8 = 15;
const BG_B: u8 = 26;

/// Grid line color — rgba(59, 130, 246, 0.08)
const GRID_R: u8 = 59;
const GRID_G: u8 = 130;
const GRID_B: u8 = 246;
const GRID_ALPHA: f32 = 0.08;

/// Default stroke color (#e2e8f0)
const DEFAULT_STROKE_R: u8 = 226;
const DEFAULT_STROKE_G: u8 = 232;
const DEFAULT_STROKE_B: u8 = 240;

/// Accent / selection color (#3b82f6)
const ACCENT_R: u8 = 59;
const ACCENT_G: u8 = 130;
const ACCENT_B: u8 = 246;

/// Selection fill alpha — rgba(59, 130, 246, 0.08)
const SELECTION_FILL_ALPHA: f32 = 0.08;

/// Handle fill (#ffffff)
const HANDLE_FILL_R: u8 = 255;
const HANDLE_FILL_G: u8 = 255;
const HANDLE_FILL_B: u8 = 255;

/// Corner radius for rectangles and diamonds (px)
const CORNER_RADIUS: f32 = 12.0;

/// Arrowhead geometry
const ARROWHEAD_LENGTH: f32 = 14.0;
const ARROWHEAD_ANGLE: f32 = 0.45;

/// Hachure fill
const HACHURE_LINE_WIDTH: f32 = 1.5;
const HACHURE_ALPHA: f32 = 0.5;

/// Selection visuals
const SELECTION_PAD: f32 = 5.0;
const SELECTION_DASH_LEN: f32 = 5.0;
const HANDLE_RADIUS: f32 = 4.0;

/// Grid spacing (world units)
const GRID_SIZE: f32 = 20.0;
/// Minimum screen-space grid spacing before hiding
const GRID_MIN_SCREEN_PX: f32 = 8.0;

/// Hit-test padding (world units)
const HIT_TEST_PAD: f32 = 4.0;

/// Line proximity tolerance for hit testing lines/arrows (world units)
const LINE_HIT_TOLERANCE: f32 = 6.0;

/// Approximate character width factor for text bounds
const TEXT_CHAR_WIDTH_FACTOR: f32 = 0.6;
/// Line height factor for text bounds
const TEXT_LINE_HEIGHT_FACTOR: f32 = 1.2;
/// Minimum text width in character-equivalents
const TEXT_MIN_CHARS: f32 = 2.0;

// ── Public types ────────────────────────────────────────────────────────

/// Configuration for the renderer.
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub background: Color,
    /// Device pixel ratio (2.0 for retina)
    pub pixel_ratio: f32,
    pub show_grid: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            background: Color::from_rgba8(BG_R, BG_G, BG_B, 255),
            pixel_ratio: 1.0,
            show_grid: true,
        }
    }
}

/// Resize handle position on a selection box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlePosition {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

/// The renderer. Stateless aside from config — call `render()` each frame.
pub struct Renderer {
    config: RenderConfig,
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Access renderer configuration.
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    // ── Main render entry point ─────────────────────────────────────

    /// Render the full document to a pixel buffer.
    pub fn render(
        &self,
        doc: &Document,
        viewport: &ViewState,
        selected_ids: &[&str],
        selection_box: Option<Bounds>,
    ) -> Pixmap {
        let pw = (self.config.width as f32 * self.config.pixel_ratio) as u32;
        let ph = (self.config.height as f32 * self.config.pixel_ratio) as u32;
        let mut pixmap = Pixmap::new(pw.max(1), ph.max(1)).expect("pixmap dimensions must be > 0");

        // Background fill
        pixmap.fill(self.config.background);

        // Grid
        if self.config.show_grid {
            self.draw_grid(&mut pixmap, viewport);
        }

        // Viewport transform: screen = world * zoom + scroll
        let vt = viewport_transform(viewport, self.config.pixel_ratio);

        // Draw elements in z-order (front to back)
        for el in &doc.elements {
            self.draw_element(&mut pixmap, el, &vt);
        }

        // Selection highlights
        for el in &doc.elements {
            if selected_ids.contains(&el.id()) && !has_group_id(el) {
                self.draw_selection_box(&mut pixmap, el, viewport);
            }
        }

        // Rubber band selection rectangle (in screen coords)
        if let Some(sb) = selection_box {
            self.draw_rubber_band(&mut pixmap, &sb);
        }

        pixmap
    }

    // ── Hit testing ─────────────────────────────────────────────────

    /// Hit test: which element is at this screen point? Returns element id.
    /// Iterates in reverse z-order (topmost first).
    pub fn hit_test(
        &self,
        doc: &Document,
        viewport: &ViewState,
        screen_x: f32,
        screen_y: f32,
    ) -> Option<String> {
        let (wx, wy) = screen_to_world(viewport, screen_x, screen_y);
        for el in doc.elements.iter().rev() {
            if self.hit_test_element(el, wx, wy) {
                return Some(el.id().to_string());
            }
        }
        None
    }

    /// Hit test for resize handles. Returns (element_id, handle_position).
    pub fn hit_test_handle(
        &self,
        doc: &Document,
        viewport: &ViewState,
        screen_x: f32,
        screen_y: f32,
    ) -> Option<(String, HandlePosition)> {
        let (wx, wy) = screen_to_world(viewport, screen_x, screen_y);
        let hs = HANDLE_RADIUS / viewport.zoom as f32;

        for el in doc.elements.iter().rev() {
            if let Some(eb) = element_bounds_f32(el) {
                let handles = [
                    (HandlePosition::NorthWest, eb.x(), eb.y()),
                    (HandlePosition::NorthEast, eb.x() + eb.width(), eb.y()),
                    (HandlePosition::SouthWest, eb.x(), eb.y() + eb.height()),
                    (
                        HandlePosition::SouthEast,
                        eb.x() + eb.width(),
                        eb.y() + eb.height(),
                    ),
                ];
                for (pos, hx, hy) in &handles {
                    if (wx - hx).abs() < hs && (wy - hy).abs() < hs {
                        return Some((el.id().to_string(), *pos));
                    }
                }
            }
        }
        None
    }

    /// Get element ids within a selection rectangle (world coords in Bounds).
    pub fn elements_in_rect(
        &self,
        doc: &Document,
        _viewport: &ViewState,
        rect: Bounds,
    ) -> Vec<String> {
        let sel = to_skia_rect_from_bounds(&rect);
        let mut result = Vec::new();
        if let Some(sel) = sel {
            for el in &doc.elements {
                if let Some(eb) = element_bounds_f32(el)
                    && rects_intersect(&sel, &eb)
                {
                    result.push(el.id().to_string());
                }
            }
        }
        result
    }

    // ── Element drawing ─────────────────────────────────────────────

    fn draw_element(&self, pixmap: &mut Pixmap, el: &Element, transform: &Transform) {
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
        let x = if el.width < 0.0 {
            el.x + el.width
        } else {
            el.x
        } as f32;
        let y = if el.height < 0.0 {
            el.y + el.height
        } else {
            el.y
        } as f32;
        let w = (el.width).abs() as f32;
        let h = (el.height).abs() as f32;
        if w < 0.5 && h < 0.5 {
            return;
        }

        let radius = CORNER_RADIUS.min(w / 3.0).min(h / 3.0);
        if let Some(path) = build_rounded_rect_path(x, y, w, h, radius) {
            // Fill
            self.fill_shape(pixmap, &path, &el.fill, x, y, w, h, transform, opacity);
            // Stroke
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
            let bx = if el.width < 0.0 {
                el.x + el.width
            } else {
                el.x
            } as f32;
            let by = if el.height < 0.0 {
                el.y + el.height
            } else {
                el.y
            } as f32;
            let w = (el.width).abs() as f32;
            let h = (el.height).abs() as f32;

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
        let x = if el.width < 0.0 {
            el.x + el.width
        } else {
            el.x
        } as f32;
        let y = if el.height < 0.0 {
            el.y + el.height
        } else {
            el.y
        } as f32;
        let w = (el.width).abs() as f32;
        let h = (el.height).abs() as f32;
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
        // Draw the line portion
        self.draw_line(pixmap, el, transform, opacity);
        if el.points.len() < 2 {
            return;
        }

        // Arrowhead
        let last = el.points.last().unwrap();
        let prev = &el.points[el.points.len() - 2];
        let tip_x = last.x as f32 + el.x as f32;
        let tip_y = last.y as f32 + el.y as f32;
        let angle = (last.y as f32 - prev.y as f32).atan2(last.x as f32 - prev.x as f32);

        let left_x = tip_x - ARROWHEAD_LENGTH * (angle - ARROWHEAD_ANGLE).cos();
        let left_y = tip_y - ARROWHEAD_LENGTH * (angle - ARROWHEAD_ANGLE).sin();
        let right_x = tip_x - ARROWHEAD_LENGTH * (angle + ARROWHEAD_ANGLE).cos();
        let right_y = tip_y - ARROWHEAD_LENGTH * (angle + ARROWHEAD_ANGLE).sin();

        let mut pb = PathBuilder::new();
        pb.move_to(tip_x, tip_y);
        pb.line_to(left_x, left_y);
        pb.line_to(right_x, right_y);
        pb.close();
        if let Some(path) = pb.finish() {
            // Filled arrowhead
            let color = parse_color(&el.stroke.color, opacity);
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.fill_path(&path, &paint, FillRule::Winding, *transform, None);

            // Thin stroke around arrowhead
            let stroke = Stroke {
                width: (el.stroke.width as f32) * 0.5,
                line_cap: LineCap::Round,
                line_join: LineJoin::Round,
                ..Stroke::default()
            };
            pixmap.stroke_path(&path, &paint, &stroke, *transform, None);
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

    /// TODO: Text is rendered as a colored placeholder rectangle. Full glyph
    /// rendering via cosmic-text will be added in a follow-up phase.
    fn draw_text(
        &self,
        pixmap: &mut Pixmap,
        el: &TextElement,
        transform: &Transform,
        opacity: f32,
    ) {
        let size = el.font.size as f32;
        let lines: Vec<&str> = el.text.split('\n').collect();
        let max_chars = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as f32;
        let w = (max_chars * size * TEXT_CHAR_WIDTH_FACTOR).max(size * TEXT_MIN_CHARS);
        let h = (lines.len() as f32 * size * TEXT_LINE_HEIGHT_FACTOR)
            .max(size * TEXT_LINE_HEIGHT_FACTOR);
        let x = el.x as f32;
        let y = el.y as f32;

        // Draw placeholder rectangle with text color at reduced opacity
        let color = parse_color(&el.stroke.color, opacity * 0.3);
        let rect = Rect::from_xywh(x, y, w.max(1.0), h.max(1.0));
        if let Some(rect) = rect {
            let mut paint = Paint::default();
            paint.set_color(color);
            paint.anti_alias = true;
            pixmap.fill_rect(rect, &paint, *transform, None);

            // Dashed border to indicate placeholder
            let border_color = parse_color(&el.stroke.color, opacity * 0.5);
            let mut border_paint = Paint::default();
            border_paint.set_color(border_color);
            border_paint.anti_alias = true;
            let stroke = Stroke {
                width: 1.0,
                dash: StrokeDash::new(vec![3.0, 3.0], 0.0),
                ..Stroke::default()
            };
            let mut pb = PathBuilder::new();
            pb.push_rect(rect);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &border_paint, &stroke, *transform, None);
            }
        }
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
                // Second pass perpendicular
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

    /// Draw hachure (diagonal parallel lines) clipped to the shape path.
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
        // Build a clip mask from the shape
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

        // Generate hachure lines rotated around shape center
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;
        let diag = (w * w + h * h).sqrt();

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let mut d = -diag;
        while d < diag {
            // Line endpoints rotated around (cx, cy)
            let x1 = cx + d * cos_a - (-diag) * sin_a;
            let y1 = cy + d * sin_a + (-diag) * cos_a;
            let x2 = cx + d * cos_a - diag * sin_a;
            let y2 = cy + d * sin_a + diag * cos_a;

            let mut pb = PathBuilder::new();
            pb.move_to(x1, y1);
            pb.line_to(x2, y2);
            if let Some(line_path) = pb.finish() {
                pixmap.stroke_path(&line_path, &paint, &stroke, *transform, Some(&clip_mask));
            }
            d += gap;
        }
    }

    // ── Grid ────────────────────────────────────────────────────────

    fn draw_grid(&self, pixmap: &mut Pixmap, viewport: &ViewState) {
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

        // Vertical lines
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

        // Horizontal lines
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

    // ── Selection visuals ───────────────────────────────────────────

    fn draw_selection_box(&self, pixmap: &mut Pixmap, el: &Element, viewport: &ViewState) {
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

    fn draw_rubber_band(&self, pixmap: &mut Pixmap, sb: &Bounds) {
        let pr = self.config.pixel_ratio;
        let x = sb.x as f32 * pr;
        let y = sb.y as f32 * pr;
        let w = (sb.width as f32 * pr).abs().max(1.0);
        let h = (sb.height as f32 * pr).abs().max(1.0);

        if let Some(rect) = Rect::from_xywh(x, y, w, h) {
            let identity = Transform::identity();

            // Fill
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

            // Dashed stroke
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

    // ── Hit testing internals ───────────────────────────────────────

    fn hit_test_element(&self, el: &Element, wx: f32, wy: f32) -> bool {
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

// ── Path builders ───────────────────────────────────────────────────────

fn build_rounded_rect_path(x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<Path> {
    let r = radius.min(w / 2.0).min(h / 2.0);
    let mut pb = PathBuilder::new();

    if r <= 0.5 {
        let rect = Rect::from_xywh(x, y, w, h)?;
        pb.push_rect(rect);
    } else {
        // Top edge
        pb.move_to(x + r, y);
        pb.line_to(x + w - r, y);
        // Top-right corner
        pb.quad_to(x + w, y, x + w, y + r);
        // Right edge
        pb.line_to(x + w, y + h - r);
        // Bottom-right corner
        pb.quad_to(x + w, y + h, x + w - r, y + h);
        // Bottom edge
        pb.line_to(x + r, y + h);
        // Bottom-left corner
        pb.quad_to(x, y + h, x, y + h - r);
        // Left edge
        pb.line_to(x, y + r);
        // Top-left corner
        pb.quad_to(x, y, x + r, y);
        pb.close();
    }
    pb.finish()
}

fn build_ellipse_path(cx: f32, cy: f32, rx: f32, ry: f32) -> Option<Path> {
    // Approximate ellipse with 4 cubic bezier segments
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

fn build_diamond_path(x: f32, y: f32, w: f32, h: f32) -> Option<Path> {
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

    // Start between left and top
    pb.move_to(lerp_f32(left.0, top.0, t_lt), lerp_f32(left.1, top.1, t_lt));
    // Round corner at top
    pb.quad_to(
        top.0,
        top.1,
        lerp_f32(top.0, right.0, t_tr),
        lerp_f32(top.1, right.1, t_tr),
    );
    // Round corner at right
    pb.quad_to(
        right.0,
        right.1,
        lerp_f32(right.0, bottom.0, t_rb),
        lerp_f32(right.1, bottom.1, t_rb),
    );
    // Round corner at bottom
    pb.quad_to(
        bottom.0,
        bottom.1,
        lerp_f32(bottom.0, left.0, t_bl),
        lerp_f32(bottom.1, left.1, t_bl),
    );
    // Round corner at left
    pb.quad_to(
        left.0,
        left.1,
        lerp_f32(left.0, top.0, t_lt),
        lerp_f32(left.1, top.1, t_lt),
    );
    pb.close();
    pb.finish()
}

fn build_polyline_path(el: &LineElement) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let first = el.points.first()?;
    pb.move_to(first.x as f32 + el.x as f32, first.y as f32 + el.y as f32);
    for p in &el.points[1..] {
        pb.line_to(p.x as f32 + el.x as f32, p.y as f32 + el.y as f32);
    }
    pb.finish()
}

fn build_freedraw_path(el: &FreeDrawElement) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let ox = el.x as f32;
    let oy = el.y as f32;
    let pts = &el.points;

    pb.move_to(pts[0].x as f32 + ox, pts[0].y as f32 + oy);
    // Smooth bezier interpolation through midpoints
    for i in 1..pts.len().saturating_sub(1) {
        let xc = (pts[i].x as f32 + pts[i + 1].x as f32) / 2.0 + ox;
        let yc = (pts[i].y as f32 + pts[i + 1].y as f32) / 2.0 + oy;
        pb.quad_to(pts[i].x as f32 + ox, pts[i].y as f32 + oy, xc, yc);
    }
    let last = pts.last()?;
    pb.line_to(last.x as f32 + ox, last.y as f32 + oy);
    pb.finish()
}

fn build_circle_path(cx: f32, cy: f32, r: f32) -> Option<Path> {
    build_ellipse_path(cx, cy, r, r)
}

// ── Color parsing ───────────────────────────────────────────────────────

/// Parse a CSS hex color string (e.g. "#3b82f6") into a tiny-skia Color.
/// Falls back to default stroke color on parse failure.
fn parse_color(hex: &str, opacity: f32) -> Color {
    let hex = hex.trim().trim_start_matches('#');
    let (r, g, b) = match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(DEFAULT_STROKE_R);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(DEFAULT_STROKE_G);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(DEFAULT_STROKE_B);
            (r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16)
                .map(|v| v * 17)
                .unwrap_or(DEFAULT_STROKE_R);
            let g = u8::from_str_radix(&hex[1..2], 16)
                .map(|v| v * 17)
                .unwrap_or(DEFAULT_STROKE_G);
            let b = u8::from_str_radix(&hex[2..3], 16)
                .map(|v| v * 17)
                .unwrap_or(DEFAULT_STROKE_B);
            (r, g, b)
        }
        _ => (DEFAULT_STROKE_R, DEFAULT_STROKE_G, DEFAULT_STROKE_B),
    };
    let a = (opacity * 255.0).clamp(0.0, 255.0) as u8;
    Color::from_rgba8(r, g, b, a)
}

// ── Stroke helper ───────────────────────────────────────────────────────

fn stroke_from_style(style: &StrokeStyle, opacity: f32) -> (Paint<'static>, Stroke) {
    let color = parse_color(&style.color, opacity);
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let dash = if style.dash.is_empty() {
        None
    } else {
        let dash_vals: Vec<f32> = style.dash.iter().map(|d| *d as f32).collect();
        StrokeDash::new(dash_vals, 0.0)
    };

    let stroke = Stroke {
        width: style.width as f32,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        dash,
        ..Stroke::default()
    };
    (paint, stroke)
}

// ── Viewport transform ──────────────────────────────────────────────────

/// Build the viewport transform: screen = world * zoom + scroll.
/// Incorporates pixel_ratio scaling.
fn viewport_transform(viewport: &ViewState, pixel_ratio: f32) -> Transform {
    let pr = pixel_ratio;
    let zoom = viewport.zoom as f32 * pr;
    let tx = viewport.scroll_x as f32 * pr;
    let ty = viewport.scroll_y as f32 * pr;
    Transform::from_row(zoom, 0.0, 0.0, zoom, tx, ty)
}

/// Convert screen coordinates to world coordinates.
fn screen_to_world(viewport: &ViewState, sx: f32, sy: f32) -> (f32, f32) {
    let wx = (sx - viewport.scroll_x as f32) / viewport.zoom as f32;
    let wy = (sy - viewport.scroll_y as f32) / viewport.zoom as f32;
    (wx, wy)
}

// ── Hit testing helpers ─────────────────────────────────────────────────

fn hit_test_shape_bounds(e: &ShapeElement, wx: f32, wy: f32) -> bool {
    let x = if e.width < 0.0 { e.x + e.width } else { e.x } as f32;
    let y = if e.height < 0.0 { e.y + e.height } else { e.y } as f32;
    let w = (e.width).abs() as f32;
    let h = (e.height).abs() as f32;
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
    let x = if e.width < 0.0 { e.x + e.width } else { e.x } as f32;
    let y = if e.height < 0.0 { e.y + e.height } else { e.y } as f32;
    let w = (e.width).abs() as f32;
    let h = (e.height).abs() as f32;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    // Manhattan distance normalized to diamond half-widths
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
    let b = crate::Element::FreeDraw(e.clone()).bounds();
    wx >= b.x as f32 - HIT_TEST_PAD
        && wx <= (b.x + b.width) as f32 + HIT_TEST_PAD
        && wy >= b.y as f32 - HIT_TEST_PAD
        && wy <= (b.y + b.height) as f32 + HIT_TEST_PAD
}

fn hit_test_text_bounds(e: &TextElement, wx: f32, wy: f32) -> bool {
    let b = crate::Element::Text(e.clone()).bounds();
    wx >= b.x as f32 - HIT_TEST_PAD
        && wx <= (b.x + b.width) as f32 + HIT_TEST_PAD
        && wy >= b.y as f32 - HIT_TEST_PAD
        && wy <= (b.y + b.height) as f32 + HIT_TEST_PAD
}

/// Distance from point (px, py) to line segment (ax,ay)-(bx,by).
fn point_to_segment_distance(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
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

// ── Geometry helpers ────────────────────────────────────────────────────

fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn dist_f32(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt()
}

/// Check if element has a group_id (bound text elements skip selection highlight).
fn has_group_id(el: &Element) -> bool {
    match el {
        Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => e.group_id.is_some(),
        Element::Line(e) | Element::Arrow(e) => e.group_id.is_some(),
        Element::FreeDraw(e) => e.group_id.is_some(),
        Element::Text(e) => e.group_id.is_some(),
    }
}

/// Convert a Bounds to a tiny_skia Rect.
fn to_skia_rect_from_bounds(b: &Bounds) -> Option<Rect> {
    Rect::from_xywh(
        b.x as f32,
        b.y as f32,
        (b.width as f32).max(0.1),
        (b.height as f32).max(0.1),
    )
}

/// Get element bounds as a tiny_skia Rect.
fn element_bounds_f32(el: &Element) -> Option<Rect> {
    let b = el.bounds();
    Rect::from_xywh(
        b.x as f32,
        b.y as f32,
        (b.width as f32).max(0.1),
        (b.height as f32).max(0.1),
    )
}

/// Check if two rects intersect.
fn rects_intersect(a: &Rect, b: &Rect) -> bool {
    a.x() < b.x() + b.width()
        && a.x() + a.width() > b.x()
        && a.y() < b.y() + b.height()
        && a.y() + a.height() > b.y()
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{FreeDrawElement, LineElement, ShapeElement, TextElement};
    use crate::point::Point;
    use crate::style::FillStyle;

    fn make_doc() -> Document {
        Document::new("test".to_string())
    }

    fn default_viewport() -> ViewState {
        ViewState::default()
    }

    fn small_config() -> RenderConfig {
        RenderConfig {
            width: 200,
            height: 200,
            ..RenderConfig::default()
        }
    }

    // ── render() produces a non-empty pixmap ────────────────────────

    #[test]
    fn test_render_empty_document_produces_background() {
        let mut config = small_config();
        config.show_grid = false; // disable grid so pixels are pure background
        let renderer = Renderer::new(config);
        let doc = make_doc();
        let vp = default_viewport();
        let pixmap = renderer.render(&doc, &vp, &[], None);
        assert_eq!(pixmap.width(), 200);
        assert_eq!(pixmap.height(), 200);

        // Check that pixels match background color (no grid overlay)
        let data = pixmap.data();
        assert_eq!(data[0], BG_R);
        assert_eq!(data[1], BG_G);
        assert_eq!(data[2], BG_B);
        assert_eq!(data[3], 255);
    }

    #[test]
    fn test_render_with_elements_not_empty() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            10.0,
            10.0,
            80.0,
            60.0,
        )));
        let vp = default_viewport();
        let pixmap = renderer.render(&doc, &vp, &[], None);
        assert!(pixmap.width() > 0);
        assert!(pixmap.height() > 0);
        // The pixmap data should differ from a pure background fill
        let bg_renderer = Renderer::new(small_config());
        let bg_pixmap = bg_renderer.render(&make_doc(), &vp, &[], None);
        assert_ne!(pixmap.data(), bg_pixmap.data());
    }

    // ── Each element type renders without panicking ─────────────────

    #[test]
    fn test_render_rectangle() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            5.0,
            5.0,
            50.0,
            40.0,
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_ellipse() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Ellipse(ShapeElement::new(
            "e1".into(),
            10.0,
            10.0,
            60.0,
            40.0,
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_diamond() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Diamond(ShapeElement::new(
            "d1".into(),
            10.0,
            10.0,
            60.0,
            60.0,
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_line() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Line(LineElement::new(
            "l1".into(),
            0.0,
            0.0,
            vec![Point::new(10.0, 10.0), Point::new(90.0, 90.0)],
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_arrow() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Arrow(LineElement::new(
            "a1".into(),
            0.0,
            0.0,
            vec![Point::new(10.0, 10.0), Point::new(90.0, 50.0)],
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_freedraw() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::FreeDraw(FreeDrawElement::new(
            "fd1".into(),
            0.0,
            0.0,
            vec![
                Point::new(10.0, 10.0),
                Point::new(20.0, 30.0),
                Point::new(40.0, 20.0),
                Point::new(60.0, 50.0),
            ],
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_text_placeholder() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Text(TextElement::new(
            "t1".into(),
            10.0,
            10.0,
            "Hello world".into(),
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    // ── Fill patterns don't panic ───────────────────────────────────

    #[test]
    fn test_render_solid_fill() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        let mut rect = ShapeElement::new("r1".into(), 10.0, 10.0, 80.0, 60.0);
        rect.fill = FillStyle {
            style: FillType::Solid,
            ..FillStyle::default()
        };
        doc.add_element(Element::Rectangle(rect));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_hachure_fill() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        let mut rect = ShapeElement::new("r1".into(), 10.0, 10.0, 80.0, 60.0);
        rect.fill = FillStyle {
            style: FillType::Hachure,
            ..FillStyle::default()
        };
        doc.add_element(Element::Rectangle(rect));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    #[test]
    fn test_render_crosshatch_fill() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        let mut rect = ShapeElement::new("r1".into(), 10.0, 10.0, 80.0, 60.0);
        rect.fill = FillStyle {
            style: FillType::CrossHatch,
            ..FillStyle::default()
        };
        doc.add_element(Element::Rectangle(rect));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }

    // ── Hit testing ─────────────────────────────────────────────────

    #[test]
    fn test_hit_test_rectangle() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            50.0,
            50.0,
            100.0,
            80.0,
        )));
        let vp = default_viewport();

        // Inside
        let hit = renderer.hit_test(&doc, &vp, 80.0, 80.0);
        assert_eq!(hit, Some("r1".into()));

        // Outside
        let miss = renderer.hit_test(&doc, &vp, 5.0, 5.0);
        assert!(miss.is_none());
    }

    #[test]
    fn test_hit_test_ellipse() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Ellipse(ShapeElement::new(
            "e1".into(),
            50.0,
            50.0,
            100.0,
            60.0,
        )));
        let vp = default_viewport();

        // Center
        let hit = renderer.hit_test(&doc, &vp, 100.0, 80.0);
        assert_eq!(hit, Some("e1".into()));
    }

    #[test]
    fn test_hit_test_line() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Line(LineElement::new(
            "l1".into(),
            0.0,
            0.0,
            vec![Point::new(10.0, 10.0), Point::new(100.0, 100.0)],
        )));
        let vp = default_viewport();

        // Near the line (midpoint)
        let hit = renderer.hit_test(&doc, &vp, 55.0, 55.0);
        assert_eq!(hit, Some("l1".into()));

        // Far from line
        let miss = renderer.hit_test(&doc, &vp, 10.0, 100.0);
        assert!(miss.is_none());
    }

    #[test]
    fn test_hit_test_returns_topmost() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r_bottom".into(),
            10.0,
            10.0,
            100.0,
            100.0,
        )));
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r_top".into(),
            20.0,
            20.0,
            80.0,
            80.0,
        )));
        let vp = default_viewport();

        // Overlapping region should return top element
        let hit = renderer.hit_test(&doc, &vp, 50.0, 50.0);
        assert_eq!(hit, Some("r_top".into()));
    }

    // ── Viewport transform ──────────────────────────────────────────

    #[test]
    fn test_screen_to_world_identity() {
        let vp = default_viewport();
        let (wx, wy) = screen_to_world(&vp, 100.0, 200.0);
        assert!((wx - 100.0).abs() < 0.01);
        assert!((wy - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_screen_to_world_with_zoom() {
        let vp = ViewState {
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: 2.0,
        };
        let (wx, wy) = screen_to_world(&vp, 100.0, 200.0);
        assert!((wx - 50.0).abs() < 0.01);
        assert!((wy - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_screen_to_world_with_scroll() {
        let vp = ViewState {
            scroll_x: 50.0,
            scroll_y: 30.0,
            zoom: 1.0,
        };
        let (wx, wy) = screen_to_world(&vp, 100.0, 80.0);
        assert!((wx - 50.0).abs() < 0.01);
        assert!((wy - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_viewport_transform_identity() {
        let vp = default_viewport();
        let t = viewport_transform(&vp, 1.0);
        assert!((t.sx - 1.0).abs() < 0.01);
        assert!((t.sy - 1.0).abs() < 0.01);
        assert!(t.tx.abs() < 0.01);
        assert!(t.ty.abs() < 0.01);
    }

    #[test]
    fn test_viewport_transform_with_zoom_and_scroll() {
        let vp = ViewState {
            scroll_x: 100.0,
            scroll_y: 50.0,
            zoom: 2.0,
        };
        let t = viewport_transform(&vp, 1.0);
        assert!((t.sx - 2.0).abs() < 0.01);
        assert!((t.tx - 100.0).abs() < 0.01);
        assert!((t.ty - 50.0).abs() < 0.01);
    }

    // ── elements_in_rect ────────────────────────────────────────────

    #[test]
    fn test_elements_in_rect() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            10.0,
            10.0,
            30.0,
            30.0,
        )));
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r2".into(),
            100.0,
            100.0,
            30.0,
            30.0,
        )));
        let vp = default_viewport();

        // Selection that covers r1 but not r2
        let ids = renderer.elements_in_rect(&doc, &vp, Bounds::new(0.0, 0.0, 50.0, 50.0));
        assert!(ids.contains(&"r1".to_string()));
        assert!(!ids.contains(&"r2".to_string()));
    }

    // ── Color parsing ───────────────────────────────────────────────

    #[test]
    fn test_parse_color_hex6() {
        let c = parse_color("#3b82f6", 1.0);
        assert_eq!(c, Color::from_rgba8(0x3b, 0x82, 0xf6, 255));
    }

    #[test]
    fn test_parse_color_hex3() {
        let c = parse_color("#fff", 1.0);
        assert_eq!(c, Color::from_rgba8(255, 255, 255, 255));
    }

    #[test]
    fn test_parse_color_with_opacity() {
        let c = parse_color("#ffffff", 0.5);
        assert_eq!(c, Color::from_rgba8(255, 255, 255, 127));
    }

    #[test]
    fn test_parse_color_invalid_fallback() {
        let c = parse_color("not-a-color", 1.0);
        assert_eq!(
            c,
            Color::from_rgba8(DEFAULT_STROKE_R, DEFAULT_STROKE_G, DEFAULT_STROKE_B, 255)
        );
    }

    // ── Geometry helpers ────────────────────────────────────────────

    #[test]
    fn test_point_to_segment_distance() {
        // Point on segment
        let d = point_to_segment_distance(5.0, 5.0, 0.0, 0.0, 10.0, 10.0);
        assert!(d < 0.01);

        // Point off to the side
        let d = point_to_segment_distance(0.0, 10.0, 0.0, 0.0, 10.0, 0.0);
        assert!((d - 10.0).abs() < 0.01);
    }

    // ── Selection rendering doesn't panic ───────────────────────────

    #[test]
    fn test_render_with_selection() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            10.0,
            10.0,
            80.0,
            60.0,
        )));
        let vp = default_viewport();
        let _ = renderer.render(&doc, &vp, &["r1"], None);
    }

    #[test]
    fn test_render_with_rubber_band() {
        let renderer = Renderer::new(small_config());
        let doc = make_doc();
        let vp = default_viewport();
        let sb = Bounds::new(10.0, 10.0, 100.0, 80.0);
        let _ = renderer.render(&doc, &vp, &[], Some(sb));
    }

    // ── Negative dimensions (drag from bottom-right to top-left) ────

    #[test]
    fn test_render_negative_dimensions() {
        let renderer = Renderer::new(small_config());
        let mut doc = make_doc();
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".into(),
            100.0,
            100.0,
            -50.0,
            -30.0,
        )));
        let _ = renderer.render(&doc, &default_viewport(), &[], None);
    }
}
