//! Sample gallery: builds 5 canonical drawings demonstrating the full
//! `draw-core` API surface and exports each as `.draw.json`, `.svg`, and
//! `.png` under `examples/gallery/`.
//!
//! Run with: `cargo run --example gallery -p dkdc-draw-core`
//!
//! Each drawing here targets a different feature slice:
//!   - flowchart:   rectangles, diamond, arrows with arrowheads, text
//!   - sticky:      solid-fill rectangles with text labels (no hachure)
//!   - wireframe:   ellipses, dashed strokes, bordered layout
//!   - sketch:      freedraw curves + text annotations
//!   - patterns:    every FillType side by side (Solid, Hachure, CrossHatch, None)

use std::fs;
use std::path::{Path, PathBuf};

use draw_core::style::{Arrowhead, FillStyle, FillType, FontStyle, StrokeStyle, TextAlign};
use draw_core::{
    Document, Element, FreeDrawElement, LineElement, Point, ShapeElement, TextElement,
    export_png_with_scale, export_svg, storage,
};

fn main() -> anyhow::Result<()> {
    let out_dir: PathBuf = std::env::var("DRAW_GALLERY_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("examples")
                .join("gallery")
        });
    fs::create_dir_all(&out_dir)?;

    let drawings = [
        ("flowchart", flowchart()),
        ("sticky", sticky_notes()),
        ("wireframe", wireframe()),
        ("sketch", sketch()),
        ("patterns", patterns()),
    ];

    for (slug, mut doc) in drawings.iter().cloned() {
        // Pin id and timestamps so regenerated artifacts are byte-identical to
        // the committed ones — reviewers see a clean no-op diff unless content
        // actually changed.
        pin_metadata(&mut doc, slug);
        write_drawing(&out_dir, slug, &doc)?;
    }

    println!("wrote {} drawings to {}", drawings.len(), out_dir.display());
    Ok(())
}

fn pin_metadata(doc: &mut Document, slug: &str) {
    doc.id = format!("gallery-{slug}");
    let pinned = "2026-01-01T00:00:00+00:00".to_string();
    doc.created_at = pinned.clone();
    doc.modified_at = pinned;
}

fn write_drawing(out_dir: &Path, slug: &str, doc: &Document) -> anyhow::Result<()> {
    let json_path = out_dir.join(format!("{slug}.draw.json"));
    storage::save(doc, &json_path)?;

    let svg_path = out_dir.join(format!("{slug}.svg"));
    fs::write(&svg_path, export_svg(doc))?;

    // 2x scale for retina-quality gallery assets.
    let png_path = out_dir.join(format!("{slug}.png"));
    fs::write(&png_path, export_png_with_scale(doc, 2.0)?)?;

    println!("wrote {slug}: json + svg + png");
    Ok(())
}

// ── Drawing builders ──────────────────────────────────────────────────

fn flowchart() -> Document {
    let mut doc = Document::new("flowchart".into());

    // Start ellipse
    doc.add_element(Element::Ellipse(shape_with_fill(
        "start",
        40.0,
        20.0,
        140.0,
        60.0,
        FillType::Hachure,
        "#10b981",
    )));
    doc.add_element(Element::Text(centered_label(
        "start-label",
        70.0,
        42.0,
        "start",
        18.0,
        "#064e3b",
    )));

    // Decision diamond
    doc.add_element(Element::Diamond(shape_with_fill(
        "decide",
        40.0,
        140.0,
        140.0,
        100.0,
        FillType::Hachure,
        "#f59e0b",
    )));
    doc.add_element(Element::Text(centered_label(
        "decide-label",
        75.0,
        180.0,
        "valid?",
        16.0,
        "#78350f",
    )));

    // Accept rectangle
    doc.add_element(Element::Rectangle(shape_with_fill(
        "accept",
        240.0,
        150.0,
        140.0,
        80.0,
        FillType::Hachure,
        "#3b82f6",
    )));
    doc.add_element(Element::Text(centered_label(
        "accept-label",
        270.0,
        180.0,
        "process",
        16.0,
        "#1e3a8a",
    )));

    // Reject rectangle
    doc.add_element(Element::Rectangle(shape_with_fill(
        "reject",
        40.0,
        300.0,
        140.0,
        80.0,
        FillType::Hachure,
        "#ef4444",
    )));
    doc.add_element(Element::Text(centered_label(
        "reject-label",
        80.0,
        330.0,
        "reject",
        16.0,
        "#7f1d1d",
    )));

    // Arrows
    doc.add_element(Element::Arrow(arrow_with_head(
        "a1",
        110.0,
        80.0,
        vec![Point::new(0.0, 0.0), Point::new(0.0, 60.0)],
    )));
    doc.add_element(Element::Arrow(arrow_with_head(
        "a2",
        180.0,
        190.0,
        vec![Point::new(0.0, 0.0), Point::new(60.0, 0.0)],
    )));
    doc.add_element(Element::Arrow(arrow_with_head(
        "a3",
        110.0,
        240.0,
        vec![Point::new(0.0, 0.0), Point::new(0.0, 60.0)],
    )));

    // Edge labels
    doc.add_element(Element::Text(centered_label(
        "yes-label",
        190.0,
        170.0,
        "yes",
        14.0,
        "#52525b",
    )));
    doc.add_element(Element::Text(centered_label(
        "no-label", 120.0, 260.0, "no", 14.0, "#52525b",
    )));

    doc
}

fn sticky_notes() -> Document {
    let mut doc = Document::new("sticky-notes".into());

    let notes = [
        (
            "note1",
            20.0,
            20.0,
            "#fef3c7",
            "#78350f",
            "shopping\n  bread\n  butter",
        ),
        (
            "note2",
            200.0,
            20.0,
            "#dbeafe",
            "#1e3a8a",
            "read list\n  Rust book\n  SICP",
        ),
        (
            "note3",
            380.0,
            20.0,
            "#fce7f3",
            "#831843",
            "weekend\n  hike\n  cook",
        ),
        (
            "note4",
            110.0,
            220.0,
            "#d1fae5",
            "#064e3b",
            "ideas\n  draw gallery\n  lint pass",
        ),
        (
            "note5",
            290.0,
            220.0,
            "#fee2e2",
            "#7f1d1d",
            "blockers\n  pixmap panic\n  hachure opacity",
        ),
    ];

    for (i, (slug, x, y, fill_hex, text_hex, text)) in notes.iter().enumerate() {
        doc.add_element(Element::Rectangle(ShapeElement {
            id: format!("{slug}-bg"),
            x: *x,
            y: *y,
            width: 160.0,
            height: 160.0,
            angle: if i % 2 == 0 { -0.03 } else { 0.03 },
            stroke: StrokeStyle {
                color: (*text_hex).to_string(),
                width: 1.5,
                dash: vec![],
            },
            fill: FillStyle {
                color: (*fill_hex).to_string(),
                style: FillType::Solid,
                gap: 10.0,
                angle: 0.0,
            },
            opacity: 1.0,
            locked: false,
            group_id: None,
        }));
        doc.add_element(Element::Text(TextElement {
            id: format!("{slug}-text"),
            x: x + 12.0,
            y: y + 16.0,
            text: (*text).to_string(),
            font: FontStyle {
                family: "Inter, sans-serif".to_string(),
                size: 14.0,
                align: TextAlign::Left,
            },
            stroke: StrokeStyle {
                color: (*text_hex).to_string(),
                width: 1.0,
                dash: vec![],
            },
            opacity: 1.0,
            angle: 0.0,
            locked: false,
            group_id: None,
        }));
    }

    doc
}

fn wireframe() -> Document {
    let mut doc = Document::new("wireframe".into());

    // Window frame
    doc.add_element(Element::Rectangle(shape_with_stroke(
        "frame",
        20.0,
        20.0,
        440.0,
        320.0,
        "#334155",
        2.5,
        vec![],
    )));
    // Title bar
    doc.add_element(Element::Rectangle(shape_with_stroke(
        "titlebar",
        20.0,
        20.0,
        440.0,
        40.0,
        "#334155",
        1.5,
        vec![],
    )));
    doc.add_element(Element::Text(centered_label(
        "title", 34.0, 34.0, "~/draw", 14.0, "#334155",
    )));

    // Dashed content area
    doc.add_element(Element::Rectangle(shape_with_stroke(
        "content",
        40.0,
        80.0,
        400.0,
        240.0,
        "#64748b",
        1.0,
        vec![6.0, 4.0],
    )));

    // Inner buttons (ellipses)
    doc.add_element(Element::Ellipse(shape_with_stroke(
        "btn1",
        60.0,
        100.0,
        80.0,
        32.0,
        "#0ea5e9",
        1.5,
        vec![],
    )));
    doc.add_element(Element::Text(centered_label(
        "btn1-label",
        80.0,
        110.0,
        "open",
        12.0,
        "#0c4a6e",
    )));

    doc.add_element(Element::Ellipse(shape_with_stroke(
        "btn2",
        160.0,
        100.0,
        80.0,
        32.0,
        "#0ea5e9",
        1.5,
        vec![],
    )));
    doc.add_element(Element::Text(centered_label(
        "btn2-label",
        180.0,
        110.0,
        "save",
        12.0,
        "#0c4a6e",
    )));

    doc.add_element(Element::Ellipse(shape_with_stroke(
        "btn3",
        260.0,
        100.0,
        80.0,
        32.0,
        "#0ea5e9",
        1.5,
        vec![],
    )));
    doc.add_element(Element::Text(centered_label(
        "btn3-label",
        280.0,
        110.0,
        "export",
        12.0,
        "#0c4a6e",
    )));

    // Canvas placeholder with crossed diagonals
    doc.add_element(Element::Rectangle(shape_with_stroke(
        "canvas",
        60.0,
        160.0,
        380.0,
        140.0,
        "#94a3b8",
        1.0,
        vec![],
    )));
    doc.add_element(Element::Line(LineElement::new(
        "d1".into(),
        60.0,
        160.0,
        vec![Point::new(0.0, 0.0), Point::new(380.0, 140.0)],
    )));
    doc.add_element(Element::Line(LineElement::new(
        "d2".into(),
        440.0,
        160.0,
        vec![Point::new(0.0, 0.0), Point::new(-380.0, 140.0)],
    )));

    doc
}

fn sketch() -> Document {
    let mut doc = Document::new("sketch".into());

    // A squiggly curve
    let squiggle: Vec<Point> = (0..60)
        .map(|i| {
            let t = i as f64;
            Point::new(t * 6.0, (t * 0.3).sin() * 40.0)
        })
        .collect();
    doc.add_element(Element::FreeDraw(FreeDrawElement {
        id: "wave".into(),
        x: 30.0,
        y: 80.0,
        points: squiggle,
        stroke: StrokeStyle {
            color: "#8b5cf6".to_string(),
            width: 2.5,
            dash: vec![],
        },
        opacity: 1.0,
        locked: false,
        group_id: None,
    }));

    // A heart-ish freedraw
    let heart: Vec<Point> = (0..64)
        .map(|i| {
            let t = i as f64 / 64.0 * std::f64::consts::TAU;
            let x = 16.0 * t.sin().powi(3);
            let y =
                -(13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos());
            Point::new(x * 3.0, y * 3.0)
        })
        .collect();
    doc.add_element(Element::FreeDraw(FreeDrawElement {
        id: "heart".into(),
        x: 250.0,
        y: 180.0,
        points: heart,
        stroke: StrokeStyle {
            color: "#e11d48".to_string(),
            width: 3.0,
            dash: vec![],
        },
        opacity: 1.0,
        locked: false,
        group_id: None,
    }));

    // Annotation arrow pointing at the heart
    doc.add_element(Element::Arrow(arrow_with_head(
        "ann-arrow",
        100.0,
        220.0,
        vec![Point::new(0.0, 0.0), Point::new(120.0, -20.0)],
    )));
    doc.add_element(Element::Text(centered_label(
        "ann-text",
        20.0,
        220.0,
        "freedraw!",
        16.0,
        "#334155",
    )));

    doc.add_element(Element::Text(centered_label(
        "wave-text",
        30.0,
        40.0,
        "sin wave via FreeDraw",
        16.0,
        "#334155",
    )));

    doc
}

fn patterns() -> Document {
    let mut doc = Document::new("fill-patterns".into());

    let tiles = [
        ("solid", 20.0, "Solid", FillType::Solid, "#ec4899"),
        ("hachure", 200.0, "Hachure", FillType::Hachure, "#ec4899"),
        (
            "crosshatch",
            380.0,
            "CrossHatch",
            FillType::CrossHatch,
            "#ec4899",
        ),
        ("none", 560.0, "None", FillType::None, "#ec4899"),
    ];

    for (slug, x, label, style, color) in tiles {
        doc.add_element(Element::Rectangle(ShapeElement {
            id: format!("{slug}-rect"),
            x,
            y: 40.0,
            width: 160.0,
            height: 120.0,
            angle: 0.0,
            stroke: StrokeStyle {
                color: color.to_string(),
                width: 2.0,
                dash: vec![],
            },
            fill: FillStyle {
                color: color.to_string(),
                style,
                gap: 8.0,
                angle: -0.785,
            },
            opacity: 1.0,
            locked: false,
            group_id: None,
        }));
        doc.add_element(Element::Text(centered_label(
            &format!("{slug}-label"),
            x + 10.0,
            180.0,
            label,
            14.0,
            "#1f2937",
        )));
    }

    doc
}

// ── Small builders ────────────────────────────────────────────────────

fn shape_with_fill(
    id: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    style: FillType,
    color: &str,
) -> ShapeElement {
    ShapeElement {
        id: id.to_string(),
        x,
        y,
        width: w,
        height: h,
        angle: 0.0,
        stroke: StrokeStyle {
            color: color.to_string(),
            width: 2.0,
            dash: vec![],
        },
        fill: FillStyle {
            color: color.to_string(),
            style,
            gap: 10.0,
            angle: -0.785,
        },
        opacity: 1.0,
        locked: false,
        group_id: None,
    }
}

fn shape_with_stroke(
    id: &str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    color: &str,
    stroke_w: f64,
    dash: Vec<f64>,
) -> ShapeElement {
    ShapeElement {
        id: id.to_string(),
        x,
        y,
        width: w,
        height: h,
        angle: 0.0,
        stroke: StrokeStyle {
            color: color.to_string(),
            width: stroke_w,
            dash,
        },
        fill: FillStyle {
            color: color.to_string(),
            style: FillType::None,
            gap: 10.0,
            angle: 0.0,
        },
        opacity: 1.0,
        locked: false,
        group_id: None,
    }
}

fn arrow_with_head(id: &str, x: f64, y: f64, points: Vec<Point>) -> LineElement {
    LineElement {
        id: id.to_string(),
        x,
        y,
        points,
        stroke: StrokeStyle {
            color: "#334155".to_string(),
            width: 2.0,
            dash: vec![],
        },
        start_arrowhead: None,
        end_arrowhead: Some(Arrowhead::Arrow),
        opacity: 1.0,
        locked: false,
        group_id: None,
        start_binding: None,
        end_binding: None,
    }
}

fn centered_label(id: &str, x: f64, y: f64, text: &str, size: f64, color: &str) -> TextElement {
    TextElement {
        id: id.to_string(),
        x,
        y,
        text: text.to_string(),
        font: FontStyle {
            family: "Inter, sans-serif".to_string(),
            size,
            align: TextAlign::Left,
        },
        stroke: StrokeStyle {
            color: color.to_string(),
            width: 1.0,
            dash: vec![],
        },
        opacity: 1.0,
        angle: 0.0,
        locked: false,
        group_id: None,
    }
}
