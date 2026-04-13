//! Integration tests exercising draw_core as an external consumer would —
//! only `pub use` re-exports from the crate root, cross-module flows,
//! and gaps in inline unit-test coverage (Binding round-trip, full pipeline).

use draw_core::{
    Binding, Document, Element, FreeDrawElement, LineElement, Point, ShapeElement, TextElement,
    export_png, export_svg,
};

/// Build a document containing one of every Element variant.
fn populated_doc(name: &str) -> Document {
    let mut doc = Document::new(name.to_string());
    doc.add_element(Element::Rectangle(ShapeElement::new(
        "rect-1".into(),
        10.0,
        10.0,
        80.0,
        40.0,
    )));
    doc.add_element(Element::Ellipse(ShapeElement::new(
        "ell-1".into(),
        120.0,
        10.0,
        60.0,
        60.0,
    )));
    doc.add_element(Element::Diamond(ShapeElement::new(
        "dia-1".into(),
        10.0,
        80.0,
        60.0,
        60.0,
    )));
    doc.add_element(Element::Line(LineElement::new(
        "line-1".into(),
        120.0,
        80.0,
        vec![Point::new(0.0, 0.0), Point::new(40.0, 40.0)],
    )));
    doc.add_element(Element::Arrow(LineElement::new(
        "arr-1".into(),
        10.0,
        160.0,
        vec![Point::new(0.0, 0.0), Point::new(60.0, 0.0)],
    )));
    doc.add_element(Element::FreeDraw(FreeDrawElement::new(
        "fd-1".into(),
        120.0,
        160.0,
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 15.0),
            Point::new(25.0, 5.0),
        ],
    )));
    doc.add_element(Element::Text(TextElement::new(
        "txt-1".into(),
        10.0,
        240.0,
        "hello draw".into(),
    )));
    doc
}

#[test]
fn full_pipeline_all_element_types() {
    let doc = populated_doc("full pipeline");
    assert_eq!(doc.elements.len(), 7);

    // JSON round-trip preserves structure
    let json = serde_json::to_string(&doc).expect("serialize");
    let loaded: Document = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(doc, loaded);

    // SVG export is well-formed and mentions every element we put in
    let svg = export_svg(&doc);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
    // Each element type produces at least one of its SVG primitive
    assert!(svg.contains("<rect"), "rectangle missing: {svg}");
    assert!(svg.contains("<ellipse"), "ellipse missing: {svg}");
    assert!(svg.contains("<polygon"), "diamond missing: {svg}");
    assert!(svg.contains("<path"), "line/arrow/freedraw missing: {svg}");
    assert!(svg.contains("<text"), "text missing: {svg}");

    // PNG export produces a valid PNG (magic bytes present)
    let png = export_png(&doc).expect("png export");
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']), "bad PNG header");
}

#[test]
fn file_save_load_roundtrip_preserves_everything() {
    use draw_core::storage;

    let doc = populated_doc("save load");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("drawing.draw.json");

    storage::save(&doc, &path).expect("save");
    let loaded = storage::load(&path).expect("load");

    assert_eq!(doc, loaded);
}

#[test]
fn arrow_bindings_roundtrip_via_serde() {
    // Bindings were added in #35 but have no inline test coverage.
    let mut arrow = LineElement::new(
        "a1".into(),
        0.0,
        0.0,
        vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)],
    );
    arrow.start_binding = Some(Binding {
        element_id: "rect-a".into(),
        focus: 0.25,
        gap: 4.0,
    });
    arrow.end_binding = Some(Binding {
        element_id: "rect-b".into(),
        focus: -0.5,
        gap: 8.0,
    });

    let mut doc = Document::new("bindings".into());
    doc.add_element(Element::Arrow(arrow));

    let json = serde_json::to_string(&doc).unwrap();
    let loaded: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(doc, loaded);

    let Element::Arrow(restored) = &loaded.elements[0] else {
        panic!("expected Arrow variant after round-trip");
    };
    let start = restored.start_binding.as_ref().expect("start_binding");
    assert_eq!(start.element_id, "rect-a");
    assert_eq!(start.focus, 0.25);
    assert_eq!(start.gap, 4.0);
    let end = restored.end_binding.as_ref().expect("end_binding");
    assert_eq!(end.element_id, "rect-b");
    assert_eq!(end.focus, -0.5);
    assert_eq!(end.gap, 8.0);
}

#[test]
fn bindings_default_to_none_on_legacy_docs() {
    // A LineElement serialized before bindings existed should still deserialize.
    let legacy_json = r#"{
        "id": "test",
        "version": 1,
        "name": "legacy",
        "elements": [{
            "type": "Arrow",
            "id": "a1",
            "x": 0.0,
            "y": 0.0,
            "points": [{"x": 0.0, "y": 0.0}, {"x": 10.0, "y": 0.0}]
        }],
        "created_at": "2026-01-01T00:00:00Z",
        "modified_at": "2026-01-01T00:00:00Z"
    }"#;
    let doc: Document = serde_json::from_str(legacy_json).expect("legacy parse");
    let Element::Arrow(line) = &doc.elements[0] else {
        panic!("expected Arrow");
    };
    assert!(line.start_binding.is_none());
    assert!(line.end_binding.is_none());
}

#[test]
fn element_helpers_dispatch_across_all_variants() {
    // Exercises Element::id() / position() / set_position() / opacity() /
    // is_locked() / group_id() for every variant via the public API.
    let mut doc = populated_doc("helpers");
    for element in &mut doc.elements {
        let original_id = element.id().to_string();
        let (x, y) = element.position();
        element.set_position(x + 5.0, y + 5.0);
        let (nx, ny) = element.position();
        assert_eq!(nx, x + 5.0, "{original_id} x did not move");
        assert_eq!(ny, y + 5.0, "{original_id} y did not move");
        assert_eq!(element.opacity(), 1.0);
        assert!(!element.is_locked());
        assert!(element.group_id().is_none());
    }
}

#[test]
fn empty_document_exports_safely() {
    let doc = Document::new("empty".into());
    let svg = export_svg(&doc);
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
    let png = export_png(&doc).expect("png of empty doc");
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
}
