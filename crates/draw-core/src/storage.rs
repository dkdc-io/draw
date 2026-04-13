use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::document::Document;

/// Return the directory where drawings are stored, creating it if needed.
///
/// # Errors
/// Returns an error if the platform config directory cannot be determined or
/// the directory cannot be created.
pub fn storage_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("could not determine config directory")?
        .join("draw")
        .join("drawings");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Serialize a document to `path` as pretty-printed JSON.
///
/// # Errors
/// Returns an error if JSON serialization fails or the file cannot be written.
pub fn save(doc: &Document, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(doc)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, json)?;
    Ok(())
}

/// Save a document under [`storage_dir`] as `<doc.id>.draw.json`.
///
/// # Errors
/// Returns an error if the storage directory cannot be resolved or writing fails.
pub fn save_to_storage(doc: &Document) -> Result<PathBuf> {
    let dir = storage_dir()?;
    let path = dir.join(format!("{}.draw.json", doc.id));
    save(doc, &path)?;
    Ok(path)
}

/// Load a document from a `.draw.json` file.
///
/// # Errors
/// Returns an error if the file cannot be read or the JSON is malformed.
pub fn load(path: &Path) -> Result<Document> {
    let json = fs::read_to_string(path).context("could not read drawing file")?;
    let doc: Document = serde_json::from_str(&json).context("invalid drawing file")?;
    Ok(doc)
}

/// List all drawings in [`storage_dir`] as `(name, path)` tuples, sorted by name.
///
/// # Errors
/// Returns an error if the storage directory cannot be resolved or read.
pub fn list_drawings() -> Result<Vec<(String, PathBuf)>> {
    let dir = storage_dir()?;
    let mut drawings = vec![];

    if !dir.exists() {
        return Ok(drawings);
    }

    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json")
            && path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".draw.json"))
            && let Ok(doc) = load(&path)
        {
            drawings.push((doc.name, path));
        }
    }

    drawings.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(drawings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    #[test]
    fn test_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.draw.json");

        let doc = Document::new("test drawing".to_string());
        save(&doc, &path).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(doc.id, loaded.id);
        assert_eq!(doc.name, loaded.name);
    }

    #[test]
    fn test_save_load_with_elements() {
        use crate::element::{Element, ShapeElement, TextElement};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("elements.draw.json");

        let mut doc = Document::new("with elements".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            10.0,
            20.0,
            100.0,
            50.0,
        )));
        doc.add_element(Element::Text(TextElement::new(
            "t1".to_string(),
            5.0,
            5.0,
            "hello\nworld".to_string(),
        )));
        save(&doc, &path).unwrap();

        let loaded = load(&path).unwrap();
        assert_eq!(loaded.id, doc.id);
        assert_eq!(loaded.elements.len(), 2);
        assert_eq!(loaded.elements[0].id(), "r1");
        assert_eq!(loaded.elements[1].id(), "t1");
    }
}
