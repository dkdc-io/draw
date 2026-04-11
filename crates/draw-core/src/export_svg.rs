use crate::document::Document;
use crate::element::{Element, ShapeElement};
use crate::geometry;
use crate::point::Bounds;
use crate::style::{FillStyle, FillType};

// ── Hachure rendering constants ─────────────────────────────────────
const HACHURE_OPACITY: f64 = 0.5;
const PERPENDICULAR_OFFSET: f64 = std::f64::consts::FRAC_PI_2;

pub fn export_svg(doc: &Document) -> String {
    if doc.elements.is_empty() {
        return r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#
            .to_string();
    }

    // Compute bounding box of all elements
    let bounds = compute_bounds(&doc.elements);
    let padding = 20.0;
    let x = bounds.x - padding;
    let y = bounds.y - padding;
    let w = bounds.width + padding * 2.0;
    let h = bounds.height + padding * 2.0;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x} {y} {w} {h}" width="{w}" height="{h}">"#
    );
    svg.push('\n');

    // Collect defs (clipPaths for hachure fills)
    let mut defs = String::new();
    let mut clip_id: usize = 0;

    for element in &doc.elements {
        let (element_svg, element_defs) = render_element(element, &mut clip_id);
        defs.push_str(&element_defs);
        svg.push_str(&element_svg);
        svg.push('\n');
    }

    // Insert defs block before elements if needed
    if !defs.is_empty() {
        let defs_block = format!("  <defs>\n{defs}  </defs>\n");
        let insert_pos = svg.find('\n').unwrap() + 1;
        svg.insert_str(insert_pos, &defs_block);
    }

    svg.push_str("</svg>");
    svg
}

fn compute_bounds(elements: &[Element]) -> Bounds {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for element in elements {
        let b = element.bounds();
        min_x = min_x.min(b.x);
        min_y = min_y.min(b.y);
        max_x = max_x.max(b.x + b.width);
        max_y = max_y.max(b.y + b.height);
    }

    Bounds::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn render_element(element: &Element, clip_id: &mut usize) -> (String, String) {
    match element {
        Element::Rectangle(e) => render_rectangle(e, clip_id),
        Element::Ellipse(e) => render_ellipse(e, clip_id),
        Element::Diamond(e) => render_diamond(e, clip_id),
        Element::Line(e) | Element::Arrow(e) => {
            if e.points.len() < 2 {
                return (String::new(), String::new());
            }
            let mut d = format!("M {} {}", e.points[0].x + e.x, e.points[0].y + e.y);
            for p in &e.points[1..] {
                d.push_str(&format!(" L {} {}", p.x + e.x, p.y + e.y));
            }
            let stroke = stroke_attrs(&e.stroke.color, e.stroke.width, &e.stroke.dash);
            let marker = if matches!(element, Element::Arrow(_)) {
                let mut markers = String::new();

                // End arrowhead (tip at last point, pointing away from second-to-last)
                let last = e.points.last().unwrap();
                let prev = if e.points.len() >= 2 {
                    &e.points[e.points.len() - 2]
                } else {
                    &e.points[0]
                };
                let ah = geometry::compute_arrowhead(
                    last.x + e.x,
                    last.y + e.y,
                    prev.x + e.x,
                    prev.y + e.y,
                    geometry::ARROWHEAD_LENGTH,
                    geometry::ARROWHEAD_ANGLE,
                );
                markers.push_str(&format!(
                    r#"  <polygon points="{},{} {},{} {},{}" fill="{}" stroke="none"/>"#,
                    ah.tip_x,
                    ah.tip_y,
                    ah.left_x,
                    ah.left_y,
                    ah.right_x,
                    ah.right_y,
                    e.stroke.color
                ));

                // Start arrowhead (tip at first point, pointing away from second point)
                if e.start_arrowhead.is_some() {
                    let first = &e.points[0];
                    let next = &e.points[1];
                    let ah = geometry::compute_arrowhead(
                        first.x + e.x,
                        first.y + e.y,
                        next.x + e.x,
                        next.y + e.y,
                        geometry::ARROWHEAD_LENGTH,
                        geometry::ARROWHEAD_ANGLE,
                    );
                    markers.push_str(&format!(
                        r#"  <polygon points="{},{} {},{} {},{}" fill="{}" stroke="none"/>"#,
                        ah.tip_x,
                        ah.tip_y,
                        ah.left_x,
                        ah.left_y,
                        ah.right_x,
                        ah.right_y,
                        e.stroke.color
                    ));
                }

                markers
            } else {
                String::new()
            };
            (
                format!(
                    r#"  <path d="{d}" fill="none" {stroke} opacity="{}"/>{marker}"#,
                    e.opacity
                ),
                String::new(),
            )
        }
        Element::FreeDraw(e) => {
            if e.points.is_empty() {
                return (String::new(), String::new());
            }
            let mut d = format!("M {} {}", e.points[0].x + e.x, e.points[0].y + e.y);
            for p in &e.points[1..] {
                d.push_str(&format!(" L {} {}", p.x + e.x, p.y + e.y));
            }
            let stroke = stroke_attrs(&e.stroke.color, e.stroke.width, &e.stroke.dash);
            (
                format!(
                    r#"  <path d="{d}" fill="none" {stroke} opacity="{}" stroke-linecap="round" stroke-linejoin="round"/>"#,
                    e.opacity
                ),
                String::new(),
            )
        }
        Element::Text(e) => {
            let anchor = match e.font.align {
                crate::style::TextAlign::Left => "start",
                crate::style::TextAlign::Center => "middle",
                crate::style::TextAlign::Right => "end",
            };
            let text_color = &e.stroke.color;
            (
                format!(
                    r#"  <text x="{}" y="{}" font-family="{}" font-size="{}" text-anchor="{anchor}" fill="{}" opacity="{}">{}</text>"#,
                    e.x,
                    e.y + e.font.size,
                    xml_escape(&e.font.family),
                    e.font.size,
                    xml_escape(text_color),
                    e.opacity,
                    xml_escape(&e.text)
                ),
                String::new(),
            )
        }
    }
}

// ── Shape renderers ─────────────────────────────────────────────────

fn render_rectangle(e: &ShapeElement, clip_id: &mut usize) -> (String, String) {
    let stroke = stroke_attrs(&e.stroke.color, e.stroke.width, &e.stroke.dash);

    match e.fill.style {
        FillType::Hachure | FillType::CrossHatch => {
            let id = *clip_id;
            *clip_id += 1;
            let clip_name = format!("clip-{id}");

            let clip_def = format!(
                "    <clipPath id=\"{clip_name}\">\n      <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>\n    </clipPath>\n",
                e.x, e.y, e.width, e.height
            );

            let bounds = Bounds::new(e.x, e.y, e.width, e.height);
            let hachure_lines = render_hachure_group(&clip_name, &bounds, &e.fill, e.opacity);

            let shape = format!(
                r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="none" {stroke} opacity="{}"/>"#,
                e.x, e.y, e.width, e.height, e.opacity
            );

            (format!("{hachure_lines}\n{shape}"), clip_def)
        }
        _ => {
            let fill = fill_attr(&e.fill.color, &e.fill.style);
            (
                format!(
                    r#"  <rect x="{}" y="{}" width="{}" height="{}" {fill} {stroke} opacity="{}"/>"#,
                    e.x, e.y, e.width, e.height, e.opacity
                ),
                String::new(),
            )
        }
    }
}

fn render_ellipse(e: &ShapeElement, clip_id: &mut usize) -> (String, String) {
    let cx = e.x + e.width / 2.0;
    let cy = e.y + e.height / 2.0;
    let rx = e.width / 2.0;
    let ry = e.height / 2.0;
    let stroke = stroke_attrs(&e.stroke.color, e.stroke.width, &e.stroke.dash);

    match e.fill.style {
        FillType::Hachure | FillType::CrossHatch => {
            let id = *clip_id;
            *clip_id += 1;
            let clip_name = format!("clip-{id}");

            let clip_def = format!(
                "    <clipPath id=\"{clip_name}\">\n      <ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\"/>\n    </clipPath>\n",
            );

            let bounds = Bounds::new(e.x, e.y, e.width, e.height);
            let hachure_lines = render_hachure_group(&clip_name, &bounds, &e.fill, e.opacity);

            let shape = format!(
                r#"  <ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="none" {stroke} opacity="{}"/>"#,
                e.opacity
            );

            (format!("{hachure_lines}\n{shape}"), clip_def)
        }
        _ => {
            let fill = fill_attr(&e.fill.color, &e.fill.style);
            (
                format!(
                    r#"  <ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" {fill} {stroke} opacity="{}"/>"#,
                    e.opacity
                ),
                String::new(),
            )
        }
    }
}

fn render_diamond(e: &ShapeElement, clip_id: &mut usize) -> (String, String) {
    let cx = e.x + e.width / 2.0;
    let cy = e.y + e.height / 2.0;
    let points = format!(
        "{},{} {},{} {},{} {},{}",
        cx,
        e.y,
        e.x + e.width,
        cy,
        cx,
        e.y + e.height,
        e.x,
        cy
    );
    let stroke = stroke_attrs(&e.stroke.color, e.stroke.width, &e.stroke.dash);

    match e.fill.style {
        FillType::Hachure | FillType::CrossHatch => {
            let id = *clip_id;
            *clip_id += 1;
            let clip_name = format!("clip-{id}");

            let clip_def = format!(
                "    <clipPath id=\"{clip_name}\">\n      <polygon points=\"{points}\"/>\n    </clipPath>\n",
            );

            let bounds = Bounds::new(e.x, e.y, e.width, e.height);
            let hachure_lines = render_hachure_group(&clip_name, &bounds, &e.fill, e.opacity);

            let shape = format!(
                r#"  <polygon points="{points}" fill="none" {stroke} opacity="{}"/>"#,
                e.opacity
            );

            (format!("{hachure_lines}\n{shape}"), clip_def)
        }
        _ => {
            let fill = fill_attr(&e.fill.color, &e.fill.style);
            (
                format!(
                    r#"  <polygon points="{points}" {fill} {stroke} opacity="{}"/>"#,
                    e.opacity
                ),
                String::new(),
            )
        }
    }
}

// ── Hachure line generation ─────────────────────────────────────────

/// Generate parallel hachure lines across a bounding box at the given angle.
/// Returns SVG `<line>` elements as a string. Lines span from -diag to +diag
/// (rotated around center) so they fully cover the shape before clipping.
fn generate_hachure_lines_svg(bounds: &Bounds, color: &str, gap: f64, angle: f64) -> String {
    let cx = bounds.x + bounds.width / 2.0;
    let cy = bounds.y + bounds.height / 2.0;

    let mut svg = String::new();
    for line in geometry::generate_hachure_lines(cx, cy, bounds.width, bounds.height, gap, angle) {
        svg.push_str(&format!(
            r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}" stroke-width="{}" stroke-linecap="round"/>"#,
            line.x1, line.y1, line.x2, line.y2, geometry::HACHURE_LINE_WIDTH
        ));
        svg.push('\n');
    }
    svg
}

/// Render a `<g>` group with clip-path containing hachure lines.
/// Supports both single-pass Hachure and double-pass CrossHatch.
fn render_hachure_group(clip_id: &str, bounds: &Bounds, fill: &FillStyle, opacity: f64) -> String {
    let color = &fill.color;
    let gap = fill.gap;
    let angle = fill.angle;

    let mut lines = generate_hachure_lines_svg(bounds, color, gap, angle);

    if fill.style == FillType::CrossHatch {
        lines.push_str(&generate_hachure_lines_svg(
            bounds,
            color,
            gap,
            angle + PERPENDICULAR_OFFSET,
        ));
    }

    format!(
        r#"  <g clip-path="url(#{clip_id})" opacity="{}" style="opacity:{HACHURE_OPACITY}">
{lines}  </g>"#,
        opacity
    )
}

// ── Utility functions ───────────────────────────────────────────────

fn fill_attr(color: &str, style: &FillType) -> String {
    match style {
        FillType::None => r#"fill="none""#.to_string(),
        _ => format!(r#"fill="{color}""#),
    }
}

fn stroke_attrs(color: &str, width: f64, dash: &[f64]) -> String {
    let mut s = format!(r#"stroke="{color}" stroke-width="{width}""#);
    if !dash.is_empty() {
        let dash_str: Vec<String> = dash.iter().map(|d| d.to_string()).collect();
        s.push_str(&format!(r#" stroke-dasharray="{}""#, dash_str.join(",")));
    }
    s
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::element::{Element, ShapeElement};
    use crate::style::FillType;

    #[test]
    fn test_empty_document() {
        let doc = Document::new("test".to_string());
        let svg = export_svg(&doc);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_rectangle() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            10.0,
            20.0,
            100.0,
            50.0,
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<rect"));
        assert!(svg.contains(r#"x="10""#));
        assert!(svg.contains(r#"width="100""#));
    }

    #[test]
    fn test_ellipse() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Ellipse(ShapeElement::new(
            "e1".to_string(),
            0.0,
            0.0,
            100.0,
            60.0,
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<ellipse"));
        assert!(svg.contains(r#"rx="50""#));
        assert!(svg.contains(r#"ry="30""#));
    }

    #[test]
    fn test_line() {
        use crate::element::LineElement;
        use crate::point::Point;
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Line(LineElement::new(
            "l1".to_string(),
            0.0,
            0.0,
            vec![Point::new(0.0, 0.0), Point::new(50.0, 50.0)],
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<path"));
        assert!(svg.contains(r#"fill="none""#));
    }

    #[test]
    fn test_arrow() {
        use crate::element::LineElement;
        use crate::point::Point;
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Arrow(LineElement::new(
            "a1".to_string(),
            0.0,
            0.0,
            vec![Point::new(0.0, 0.0), Point::new(50.0, 50.0)],
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<path"));
        assert!(svg.contains("<polygon")); // arrowhead
    }

    #[test]
    fn test_text() {
        use crate::element::TextElement;
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Text(TextElement::new(
            "t1".to_string(),
            10.0,
            20.0,
            "Hello <world> & \"friends\"".to_string(),
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<text"));
        assert!(svg.contains("&lt;world&gt;"));
        assert!(svg.contains("&amp;"));
        assert!(svg.contains("&quot;friends&quot;"));
    }

    #[test]
    fn test_freedraw() {
        use crate::element::FreeDrawElement;
        use crate::point::Point;
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::FreeDraw(FreeDrawElement::new(
            "fd1".to_string(),
            0.0,
            0.0,
            vec![
                Point::new(0.0, 0.0),
                Point::new(5.0, 5.0),
                Point::new(10.0, 0.0),
            ],
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<path"));
        assert!(svg.contains("stroke-linecap"));
    }

    #[test]
    fn test_diamond() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Diamond(ShapeElement::new(
            "d1".to_string(),
            0.0,
            0.0,
            100.0,
            100.0,
        )));
        let svg = export_svg(&doc);
        assert!(svg.contains("<polygon"));
    }

    #[test]
    fn test_multi_element_svg() {
        use crate::element::{FreeDrawElement, LineElement, TextElement};
        use crate::point::Point;

        let mut doc = Document::new("multi".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            0.0,
            0.0,
            100.0,
            50.0,
        )));
        doc.add_element(Element::Ellipse(ShapeElement::new(
            "e1".to_string(),
            120.0,
            0.0,
            80.0,
            60.0,
        )));
        doc.add_element(Element::Line(LineElement::new(
            "l1".to_string(),
            0.0,
            70.0,
            vec![Point::new(0.0, 0.0), Point::new(50.0, 50.0)],
        )));
        doc.add_element(Element::FreeDraw(FreeDrawElement::new(
            "fd1".to_string(),
            60.0,
            70.0,
            vec![
                Point::new(0.0, 0.0),
                Point::new(5.0, 10.0),
                Point::new(10.0, 0.0),
            ],
        )));
        doc.add_element(Element::Text(TextElement::new(
            "t1".to_string(),
            0.0,
            150.0,
            "hello".to_string(),
        )));

        let svg = export_svg(&doc);

        // Valid SVG structure
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));

        // All element types present
        assert!(svg.contains("<rect"));
        assert!(svg.contains("<ellipse"));
        assert!(svg.contains("<text"));
        // Line and freedraw both produce <path>
        assert!(svg.matches("<path").count() >= 2);

        // viewBox is set (non-empty doc)
        assert!(svg.contains("viewBox="));
    }

    #[test]
    fn test_arrow_start_and_end_arrowheads() {
        use crate::element::LineElement;
        use crate::point::Point;
        use crate::style::Arrowhead;

        let mut doc = Document::new("arrows".to_string());
        let mut arrow = LineElement::new(
            "a1".to_string(),
            0.0,
            0.0,
            vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
        );
        arrow.start_arrowhead = Some(Arrowhead::Arrow);
        arrow.end_arrowhead = Some(Arrowhead::Arrow);
        doc.add_element(Element::Arrow(arrow));

        let svg = export_svg(&doc);

        // Should have two polygon elements (one for each arrowhead)
        let polygon_count = svg.matches("<polygon").count();
        assert_eq!(
            polygon_count, 2,
            "Expected 2 arrowhead polygons (start + end), got {polygon_count}"
        );
    }

    // ── Hachure fill tests ──────────────────────────────────────────

    #[test]
    fn test_rectangle_hachure_fill() {
        let mut doc = Document::new("test".to_string());
        let mut shape = ShapeElement::new("r1".to_string(), 10.0, 20.0, 100.0, 50.0);
        shape.fill.style = FillType::Hachure;
        shape.fill.color = "#ff0000".to_string();
        doc.add_element(Element::Rectangle(shape));
        let svg = export_svg(&doc);

        // Should have a clipPath in defs
        assert!(svg.contains("<defs>"));
        assert!(svg.contains("<clipPath"));
        assert!(svg.contains("clip-0"));
        // Should have hachure lines clipped to shape
        assert!(svg.contains(r#"clip-path="url(#clip-0)""#));
        assert!(svg.contains("<line"));
        assert!(svg.contains("stroke=\"#ff0000\""));
        // Shape stroke should still render with fill="none"
        assert!(svg.contains(r#"<rect x="10" y="20" width="100" height="50" fill="none""#));
    }

    #[test]
    fn test_ellipse_hachure_fill() {
        let mut doc = Document::new("test".to_string());
        let mut shape = ShapeElement::new("e1".to_string(), 0.0, 0.0, 80.0, 60.0);
        shape.fill.style = FillType::Hachure;
        doc.add_element(Element::Ellipse(shape));
        let svg = export_svg(&doc);

        assert!(svg.contains("<clipPath"));
        assert!(svg.contains("<ellipse cx="));
        assert!(svg.contains(r#"clip-path="url(#clip-0)""#));
        assert!(svg.contains("<line"));
    }

    #[test]
    fn test_diamond_hachure_fill() {
        let mut doc = Document::new("test".to_string());
        let mut shape = ShapeElement::new("d1".to_string(), 0.0, 0.0, 100.0, 100.0);
        shape.fill.style = FillType::Hachure;
        doc.add_element(Element::Diamond(shape));
        let svg = export_svg(&doc);

        assert!(svg.contains("<clipPath"));
        assert!(svg.contains("<polygon points="));
        assert!(svg.contains(r#"clip-path="url(#clip-0)""#));
        assert!(svg.contains("<line"));
    }

    #[test]
    fn test_crosshatch_has_more_lines_than_hachure() {
        // CrossHatch should produce roughly 2x lines (two passes)
        let mut doc_hachure = Document::new("test".to_string());
        let mut shape_h = ShapeElement::new("r1".to_string(), 0.0, 0.0, 100.0, 100.0);
        shape_h.fill.style = FillType::Hachure;
        doc_hachure.add_element(Element::Rectangle(shape_h));
        let svg_hachure = export_svg(&doc_hachure);

        let mut doc_cross = Document::new("test".to_string());
        let mut shape_c = ShapeElement::new("r2".to_string(), 0.0, 0.0, 100.0, 100.0);
        shape_c.fill.style = FillType::CrossHatch;
        doc_cross.add_element(Element::Rectangle(shape_c));
        let svg_cross = export_svg(&doc_cross);

        let hachure_lines = svg_hachure.matches("<line").count();
        let cross_lines = svg_cross.matches("<line").count();
        assert!(
            cross_lines > hachure_lines,
            "CrossHatch ({cross_lines} lines) should have more lines than Hachure ({hachure_lines} lines)"
        );
    }

    #[test]
    fn test_solid_fill_no_clippath() {
        let mut doc = Document::new("test".to_string());
        let mut shape = ShapeElement::new("r1".to_string(), 0.0, 0.0, 100.0, 50.0);
        shape.fill.style = FillType::Solid;
        shape.fill.color = "#00ff00".to_string();
        doc.add_element(Element::Rectangle(shape));
        let svg = export_svg(&doc);

        // Solid fill should NOT produce clipPaths or hachure lines
        assert!(!svg.contains("<defs>"));
        assert!(!svg.contains("<clipPath"));
        assert!(!svg.contains("<line"));
        assert!(svg.contains("fill=\"#00ff00\""));
    }

    #[test]
    fn test_none_fill_no_clippath() {
        let mut doc = Document::new("test".to_string());
        let mut shape = ShapeElement::new("r1".to_string(), 0.0, 0.0, 100.0, 50.0);
        shape.fill.style = FillType::None;
        doc.add_element(Element::Rectangle(shape));
        let svg = export_svg(&doc);

        assert!(!svg.contains("<defs>"));
        assert!(!svg.contains("<line"));
        assert!(svg.contains(r#"fill="none""#));
    }

    #[test]
    fn test_multiple_hachure_shapes_unique_clip_ids() {
        let mut doc = Document::new("test".to_string());
        let mut r1 = ShapeElement::new("r1".to_string(), 0.0, 0.0, 50.0, 50.0);
        r1.fill.style = FillType::Hachure;
        let mut r2 = ShapeElement::new("r2".to_string(), 100.0, 0.0, 50.0, 50.0);
        r2.fill.style = FillType::Hachure;
        doc.add_element(Element::Rectangle(r1));
        doc.add_element(Element::Rectangle(r2));
        let svg = export_svg(&doc);

        // Should have two distinct clip IDs
        assert!(svg.contains("clip-0"));
        assert!(svg.contains("clip-1"));
    }

    #[test]
    fn test_hachure_respects_gap() {
        let mut doc_small = Document::new("test".to_string());
        let mut shape_small = ShapeElement::new("r1".to_string(), 0.0, 0.0, 100.0, 100.0);
        shape_small.fill.style = FillType::Hachure;
        shape_small.fill.gap = 5.0;
        doc_small.add_element(Element::Rectangle(shape_small));
        let svg_small = export_svg(&doc_small);

        let mut doc_large = Document::new("test".to_string());
        let mut shape_large = ShapeElement::new("r2".to_string(), 0.0, 0.0, 100.0, 100.0);
        shape_large.fill.style = FillType::Hachure;
        shape_large.fill.gap = 20.0;
        doc_large.add_element(Element::Rectangle(shape_large));
        let svg_large = export_svg(&doc_large);

        let small_lines = svg_small.matches("<line").count();
        let large_lines = svg_large.matches("<line").count();
        assert!(
            small_lines > large_lines,
            "Smaller gap ({small_lines} lines) should produce more lines than larger gap ({large_lines} lines)"
        );
    }
}
