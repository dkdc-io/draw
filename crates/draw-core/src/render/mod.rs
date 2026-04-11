//! Unified tiny-skia renderer for both native and WASM targets.
//!
//! Split into submodules:
//! - `draw`: element rendering, fill patterns, grid
//! - `selection`: selection box and rubber band visuals
//! - `hit_test`: hit testing for elements, handles, and selection rects
//! - `path`: tiny-skia path builders for shapes

mod draw;
mod hit_test;
mod path;
mod selection;

use tiny_skia::*;

use crate::Document;
use crate::geometry;
use crate::point::{Bounds, ViewState};
use crate::style::StrokeStyle;

use hit_test::{element_bounds_f32, rects_intersect, to_skia_rect_from_bounds};

// ── Theme constants (matches frontend/theme.js) ────────────────────────

const BG_R: u8 = 10;
const BG_G: u8 = 15;
const BG_B: u8 = 26;

const GRID_R: u8 = 59;
const GRID_G: u8 = 130;
const GRID_B: u8 = 246;
const GRID_ALPHA: f32 = 0.08;

const DEFAULT_STROKE_R: u8 = 226;
const DEFAULT_STROKE_G: u8 = 232;
const DEFAULT_STROKE_B: u8 = 240;

const ACCENT_R: u8 = 59;
const ACCENT_G: u8 = 130;
const ACCENT_B: u8 = 246;

const SELECTION_FILL_ALPHA: f32 = 0.08;

const HANDLE_FILL_R: u8 = 255;
const HANDLE_FILL_G: u8 = 255;
const HANDLE_FILL_B: u8 = 255;

const CORNER_RADIUS: f32 = 12.0;

const HACHURE_LINE_WIDTH: f32 = geometry::HACHURE_LINE_WIDTH as f32;
const HACHURE_ALPHA: f32 = 0.5;

const SELECTION_PAD: f32 = 5.0;
const SELECTION_DASH_LEN: f32 = 5.0;
const HANDLE_RADIUS: f32 = 4.0;

const GRID_SIZE: f32 = 20.0;
const GRID_MIN_SCREEN_PX: f32 = 8.0;

const HIT_TEST_PAD: f32 = 4.0;
const LINE_HIT_TOLERANCE: f32 = 6.0;

const TEXT_CHAR_WIDTH_FACTOR: f32 = 0.6;
const TEXT_LINE_HEIGHT_FACTOR: f32 = 1.2;
const TEXT_MIN_CHARS: f32 = 2.0;

// ── Public types ────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub background: Color,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandlePosition {
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

/// The renderer. Stateless aside from config — call `render()` each frame.
pub struct Renderer {
    pub(self) config: RenderConfig,
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RenderConfig {
        &self.config
    }

    // ── Main render entry point ─────────────────────────────────────

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

        pixmap.fill(self.config.background);

        if self.config.show_grid {
            self.draw_grid(&mut pixmap, viewport);
        }

        let vt = viewport_transform(viewport, self.config.pixel_ratio);

        for el in &doc.elements {
            self.draw_element(&mut pixmap, el, &vt);
        }

        for el in &doc.elements {
            if selected_ids.contains(&el.id()) && el.group_id().is_none() {
                self.draw_selection_box(&mut pixmap, el, viewport);
            }
        }

        if let Some(sb) = selection_box {
            self.draw_rubber_band(&mut pixmap, &sb);
        }

        pixmap
    }

    // ── Hit testing ─────────────────────────────────────────────────

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
}

// ── Viewport helpers ───────────────────────────────────────────────────

fn viewport_transform(viewport: &ViewState, pixel_ratio: f32) -> Transform {
    let pr = pixel_ratio;
    let zoom = viewport.zoom as f32 * pr;
    let tx = viewport.scroll_x as f32 * pr;
    let ty = viewport.scroll_y as f32 * pr;
    Transform::from_row(zoom, 0.0, 0.0, zoom, tx, ty)
}

fn screen_to_world(viewport: &ViewState, sx: f32, sy: f32) -> (f32, f32) {
    let wx = (sx - viewport.scroll_x as f32) / viewport.zoom as f32;
    let wy = (sy - viewport.scroll_y as f32) / viewport.zoom as f32;
    (wx, wy)
}

// ── Color & stroke helpers ─────────────────────────────────────────────

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

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{Element, FreeDrawElement, LineElement, ShapeElement, TextElement};
    use crate::point::Point;
    use crate::style::{FillStyle, FillType};
    use hit_test::point_to_segment_distance;

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

    #[test]
    fn test_render_empty_document_produces_background() {
        let mut config = small_config();
        config.show_grid = false;
        let renderer = Renderer::new(config);
        let doc = make_doc();
        let vp = default_viewport();
        let pixmap = renderer.render(&doc, &vp, &[], None);
        assert_eq!(pixmap.width(), 200);
        assert_eq!(pixmap.height(), 200);

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
        let bg_pixmap = Renderer::new(small_config()).render(&make_doc(), &vp, &[], None);
        assert_ne!(pixmap.data(), bg_pixmap.data());
    }

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
        assert_eq!(renderer.hit_test(&doc, &vp, 80.0, 80.0), Some("r1".into()));
        assert!(renderer.hit_test(&doc, &vp, 5.0, 5.0).is_none());
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
        assert_eq!(renderer.hit_test(&doc, &vp, 100.0, 80.0), Some("e1".into()));
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
        assert_eq!(renderer.hit_test(&doc, &vp, 55.0, 55.0), Some("l1".into()));
        assert!(renderer.hit_test(&doc, &vp, 10.0, 100.0).is_none());
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
        assert_eq!(
            renderer.hit_test(&doc, &vp, 50.0, 50.0),
            Some("r_top".into())
        );
    }

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
        let ids = renderer.elements_in_rect(&doc, &vp, Bounds::new(0.0, 0.0, 50.0, 50.0));
        assert!(ids.contains(&"r1".to_string()));
        assert!(!ids.contains(&"r2".to_string()));
    }

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

    #[test]
    fn test_point_to_segment_distance() {
        let d = point_to_segment_distance(5.0, 5.0, 0.0, 0.0, 10.0, 10.0);
        assert!(d < 0.01);
        let d = point_to_segment_distance(0.0, 10.0, 0.0, 0.0, 10.0, 0.0);
        assert!((d - 10.0).abs() < 0.01);
    }

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
        let _ = renderer.render(&doc, &default_viewport(), &["r1"], None);
    }

    #[test]
    fn test_render_with_rubber_band() {
        let renderer = Renderer::new(small_config());
        let doc = make_doc();
        let sb = Bounds::new(10.0, 10.0, 100.0, 80.0);
        let _ = renderer.render(&doc, &default_viewport(), &[], Some(sb));
    }

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
