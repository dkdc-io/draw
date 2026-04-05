use crate::document::Document;
use crate::export_svg::export_svg;

/// Default scale factor for retina-quality PNG output.
const DEFAULT_SCALE: f32 = 2.0;

/// Export a document as PNG bytes at the default 2x scale.
pub fn export_png(doc: &Document) -> anyhow::Result<Vec<u8>> {
    export_png_with_scale(doc, DEFAULT_SCALE)
}

/// Export a document as PNG bytes at a given scale factor.
pub fn export_png_with_scale(doc: &Document, scale: f32) -> anyhow::Result<Vec<u8>> {
    let svg_string = export_svg(doc);

    let tree = resvg::usvg::Tree::from_str(&svg_string, &resvg::usvg::Options::default())?;
    let size = tree.size();

    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    if width == 0 || height == 0 {
        anyhow::bail!("SVG has zero dimensions, cannot render PNG");
    }

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| anyhow::anyhow!("failed to create pixmap ({width}x{height})"))?;

    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let png_bytes = pixmap.encode_png()?;
    Ok(png_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::element::{Element, ShapeElement};

    /// PNG file header magic bytes.
    const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn test_empty_document_produces_valid_png() {
        let doc = Document::new("test".to_string());
        let png = export_png(&doc).unwrap();
        assert!(!png.is_empty());
        assert!(png.starts_with(&PNG_HEADER));
    }

    #[test]
    fn test_document_with_rectangle() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            10.0,
            20.0,
            100.0,
            50.0,
        )));
        let png = export_png(&doc).unwrap();
        assert!(!png.is_empty());
        assert!(png.starts_with(&PNG_HEADER));
    }

    #[test]
    fn test_custom_scale() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            0.0,
            0.0,
            100.0,
            100.0,
        )));
        let png_1x = export_png_with_scale(&doc, 1.0).unwrap();
        let png_3x = export_png_with_scale(&doc, 3.0).unwrap();
        // 3x should produce more bytes than 1x
        assert!(png_3x.len() > png_1x.len());
    }
}
