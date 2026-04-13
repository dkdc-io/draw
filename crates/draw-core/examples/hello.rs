//! A minimal hello-world for draw-core.
//!
//! Builds a small document with a few elements, exports SVG and PNG to
//! the current directory, and prints the paths. Run with:
//!
//!     cargo run --example hello -p dkdc-draw-core

use std::fs;
use std::path::Path;

use draw_core::{
    Document, Element, FreeDrawElement, LineElement, Point, ShapeElement, TextElement, export_png,
    export_svg,
};

fn main() -> anyhow::Result<()> {
    let mut doc = Document::new("hello".into());

    doc.add_element(Element::Rectangle(ShapeElement::new(
        "box".into(),
        20.0,
        20.0,
        200.0,
        80.0,
    )));
    doc.add_element(Element::Text(TextElement::new(
        "label".into(),
        40.0,
        50.0,
        "hello, draw".into(),
    )));
    doc.add_element(Element::Arrow(LineElement::new(
        "arrow".into(),
        240.0,
        60.0,
        vec![Point::new(0.0, 0.0), Point::new(80.0, 0.0)],
    )));
    doc.add_element(Element::FreeDraw(FreeDrawElement::new(
        "squiggle".into(),
        340.0,
        60.0,
        vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, -15.0),
            Point::new(25.0, 10.0),
            Point::new(40.0, -5.0),
            Point::new(55.0, 12.0),
        ],
    )));

    let svg_path = Path::new("hello.svg");
    fs::write(svg_path, export_svg(&doc))?;
    println!("wrote {}", svg_path.display());

    let png_path = Path::new("hello.png");
    fs::write(png_path, export_png(&doc)?)?;
    println!("wrote {}", png_path.display());

    Ok(())
}
