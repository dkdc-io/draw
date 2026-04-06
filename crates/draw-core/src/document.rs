use serde::{Deserialize, Serialize};

use crate::element::Element;
use crate::point::ViewState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub version: u32,
    pub name: String,
    pub elements: Vec<Element>,
    #[serde(default)]
    pub view: ViewState,
    pub created_at: String,
    pub modified_at: String,
}

impl Document {
    pub fn new(name: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            version: 1,
            name,
            elements: vec![],
            view: ViewState::default(),
            created_at: now.clone(),
            modified_at: now,
        }
    }

    pub fn add_element(&mut self, element: Element) {
        self.touch();
        self.elements.push(element);
    }

    pub fn remove_element(&mut self, id: &str) -> Option<Element> {
        if let Some(idx) = self.elements.iter().position(|e| e.id() == id) {
            self.touch();
            Some(self.elements.remove(idx))
        } else {
            None
        }
    }

    pub fn get_element(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.id() == id)
    }

    pub fn get_element_mut(&mut self, id: &str) -> Option<&mut Element> {
        self.elements.iter_mut().find(|e| e.id() == id)
    }

    fn touch(&mut self) {
        self.modified_at = chrono::Utc::now().to_rfc3339();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{Element, ShapeElement};

    #[test]
    fn test_new_document() {
        let doc = Document::new("my drawing".to_string());
        assert_eq!(doc.name, "my drawing");
        assert_eq!(doc.version, 1);
        assert!(doc.elements.is_empty());
        assert!(!doc.id.is_empty());
    }

    #[test]
    fn test_add_element() {
        let mut doc = Document::new("test".to_string());
        let elem = Element::Rectangle(ShapeElement::new("r1".to_string(), 0.0, 0.0, 10.0, 10.0));
        doc.add_element(elem);
        assert_eq!(doc.elements.len(), 1);
        assert_eq!(doc.elements[0].id(), "r1");
    }

    #[test]
    fn test_remove_element() {
        let mut doc = Document::new("test".to_string());
        let elem = Element::Rectangle(ShapeElement::new("r1".to_string(), 0.0, 0.0, 10.0, 10.0));
        let modified_before = doc.modified_at.clone();
        doc.add_element(elem);

        // Remove existing
        let removed = doc.remove_element("r1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id(), "r1");
        assert!(doc.elements.is_empty());

        // Remove non-existing should not touch
        let modified_after_remove = doc.modified_at.clone();
        let removed = doc.remove_element("nonexistent");
        assert!(removed.is_none());
        // modified_at should not change when element not found
        assert_eq!(doc.modified_at, modified_after_remove);
        let _ = modified_before;
    }

    #[test]
    fn test_json_roundtrip() {
        use crate::element::{FreeDrawElement, LineElement, TextElement};
        use crate::point::Point;

        let mut doc = Document::new("roundtrip test".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            10.0,
            20.0,
            100.0,
            50.0,
        )));
        doc.add_element(Element::Ellipse(ShapeElement::new(
            "e1".to_string(),
            0.0,
            0.0,
            80.0,
            60.0,
        )));
        doc.add_element(Element::Line(LineElement::new(
            "l1".to_string(),
            5.0,
            5.0,
            vec![Point::new(0.0, 0.0), Point::new(50.0, 50.0)],
        )));
        doc.add_element(Element::FreeDraw(FreeDrawElement::new(
            "fd1".to_string(),
            1.0,
            2.0,
            vec![
                Point::new(0.0, 0.0),
                Point::new(3.0, 4.0),
                Point::new(6.0, 1.0),
            ],
        )));
        doc.add_element(Element::Text(TextElement::new(
            "t1".to_string(),
            0.0,
            0.0,
            "hello\nworld".to_string(),
        )));

        let json = serde_json::to_string_pretty(&doc).unwrap();
        let deserialized: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_get_element() {
        let mut doc = Document::new("test".to_string());
        doc.add_element(Element::Rectangle(ShapeElement::new(
            "r1".to_string(),
            5.0,
            10.0,
            20.0,
            30.0,
        )));
        assert!(doc.get_element("r1").is_some());
        assert!(doc.get_element("nonexistent").is_none());
    }
}
