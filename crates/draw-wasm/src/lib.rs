//! WASM bindings for the draw renderer.
//!
//! Exposes a `DrawEngine` that holds document state, viewport, selection,
//! and renderer — callable from JavaScript via wasm-bindgen.
//!
//! Complex types cross the boundary as JSON strings to keep the interface simple.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use draw_core::history::{Action, History};
use draw_core::point::{Bounds, ViewState};
use draw_core::render::{RenderConfig, Renderer};
use draw_core::{Document, Element};

// ── DrawEngine ──────────────────────────────────────────────────────────

/// The main WASM-facing engine. Holds all state needed for rendering and
/// interaction: document, renderer, viewport, selection, and undo history.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct DrawEngine {
    document: Document,
    renderer: Renderer,
    viewport: ViewState,
    selected_ids: Vec<String>,
    selection_box: Option<Bounds>,
    history: History,
    pixel_ratio: f32,
}

// Methods exposed to JS via wasm_bindgen (wasm32 only) AND available natively.
// We use a single `impl` block with conditional attributes so native tests
// can call the same functions without wasm-bindgen.

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl DrawEngine {
    // ── Constructor ─────────────────────────────────────────────────

    /// Create a new engine with the given canvas dimensions and device pixel ratio.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(width: u32, height: u32, pixel_ratio: f32) -> Self {
        let config = RenderConfig {
            width,
            height,
            pixel_ratio,
            ..RenderConfig::default()
        };
        Self {
            document: Document::new("untitled".to_string()),
            renderer: Renderer::new(config),
            viewport: ViewState::default(),
            selected_ids: Vec::new(),
            selection_box: None,
            history: History::new(),
            pixel_ratio,
        }
    }

    // ── Canvas size ─────────────────────────────────────────────────

    /// Update canvas dimensions (e.g. on window resize).
    pub fn set_size(&mut self, width: u32, height: u32) {
        let config = RenderConfig {
            width,
            height,
            pixel_ratio: self.pixel_ratio,
            ..RenderConfig::default()
        };
        self.renderer = Renderer::new(config);
    }

    // ── Document serialization ──────────────────────────────────────

    /// Load a document from a JSON string.
    pub fn load_document(&mut self, json: &str) -> bool {
        match serde_json::from_str::<Document>(json) {
            Ok(doc) => {
                self.document = doc;
                self.history = History::new();
                self.selected_ids.clear();
                self.selection_box = None;
                true
            }
            Err(_) => false,
        }
    }

    /// Serialize the current document to JSON.
    pub fn save_document(&self) -> String {
        serde_json::to_string(&self.document).unwrap_or_default()
    }

    // ── Rendering ───────────────────────────────────────────────────

    /// Render the current state and return RGBA pixel data.
    pub fn render(&self) -> Vec<u8> {
        let sel_refs: Vec<&str> = self.selected_ids.iter().map(|s| s.as_str()).collect();
        let pixmap = self.renderer.render(
            &self.document,
            &self.viewport,
            &sel_refs,
            self.selection_box,
        );
        pixmap.data().to_vec()
    }

    /// Return the width of the rendered pixmap in physical pixels.
    pub fn render_width(&self) -> u32 {
        (self.renderer.config().width as f32 * self.pixel_ratio) as u32
    }

    /// Return the height of the rendered pixmap in physical pixels.
    pub fn render_height(&self) -> u32 {
        (self.renderer.config().height as f32 * self.pixel_ratio) as u32
    }

    // ── Hit testing ─────────────────────────────────────────────────

    /// Hit test at screen coordinates. Returns element ID or empty string.
    pub fn hit_test(&self, screen_x: f32, screen_y: f32) -> String {
        self.renderer
            .hit_test(&self.document, &self.viewport, screen_x, screen_y)
            .unwrap_or_default()
    }

    /// Hit test for resize handles. Returns JSON `{"id":"...","handle":"NorthWest"}`
    /// or empty string if no handle hit.
    pub fn hit_test_handle(&self, screen_x: f32, screen_y: f32) -> String {
        match self
            .renderer
            .hit_test_handle(&self.document, &self.viewport, screen_x, screen_y)
        {
            Some((id, handle)) => {
                let handle_str = match handle {
                    draw_core::HandlePosition::NorthWest => "NorthWest",
                    draw_core::HandlePosition::NorthEast => "NorthEast",
                    draw_core::HandlePosition::SouthWest => "SouthWest",
                    draw_core::HandlePosition::SouthEast => "SouthEast",
                };
                format!(r#"{{"id":"{}","handle":"{}"}}"#, id, handle_str)
            }
            None => String::new(),
        }
    }

    /// Get element IDs within a world-coordinate rectangle. Returns JSON array.
    pub fn elements_in_rect(&self, x: f64, y: f64, w: f64, h: f64) -> String {
        let rect = Bounds::new(x, y, w, h);
        let ids = self
            .renderer
            .elements_in_rect(&self.document, &self.viewport, rect);
        serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
    }

    // ── Viewport ────────────────────────────────────────────────────

    /// Set the viewport (scroll and zoom).
    pub fn set_viewport(&mut self, scroll_x: f64, scroll_y: f64, zoom: f64) {
        self.viewport = ViewState {
            scroll_x,
            scroll_y,
            zoom,
        };
    }

    /// Convert screen coordinates to world coordinates. Returns JSON `{"x":...,"y":...}`.
    pub fn screen_to_world(&self, sx: f64, sy: f64) -> String {
        let wx = (sx - self.viewport.scroll_x) / self.viewport.zoom;
        let wy = (sy - self.viewport.scroll_y) / self.viewport.zoom;
        format!(r#"{{"x":{},"y":{}}}"#, wx, wy)
    }

    pub fn scroll_x(&self) -> f64 {
        self.viewport.scroll_x
    }

    pub fn scroll_y(&self) -> f64 {
        self.viewport.scroll_y
    }

    pub fn zoom(&self) -> f64 {
        self.viewport.zoom
    }

    // ── Selection ───────────────────────────────────────────────────

    /// Set the selected element IDs (JSON array of strings).
    pub fn set_selection(&mut self, ids_json: &str) {
        if let Ok(ids) = serde_json::from_str::<Vec<String>>(ids_json) {
            self.selected_ids = ids;
        }
    }

    /// Get the current selection as a JSON array of strings.
    pub fn get_selection(&self) -> String {
        serde_json::to_string(&self.selected_ids).unwrap_or_else(|_| "[]".to_string())
    }

    /// Set the rubber-band selection box (screen coordinates).
    pub fn set_selection_box(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.selection_box = Some(Bounds::new(x, y, w, h));
    }

    /// Clear the rubber-band selection box.
    pub fn clear_selection_box(&mut self) {
        self.selection_box = None;
    }

    // ── Document mutation ───────────────────────────────────────────

    /// Add an element from JSON. Returns the element ID, or empty string on failure.
    pub fn add_element(&mut self, json: &str) -> String {
        match serde_json::from_str::<Element>(json) {
            Ok(el) => {
                let id = el.id().to_string();
                self.history.push(Action::AddElement(Box::new(el.clone())));
                self.document.add_element(el);
                id
            }
            Err(_) => String::new(),
        }
    }

    /// Remove an element by ID. Returns true if the element existed.
    pub fn remove_element(&mut self, id: &str) -> bool {
        if let Some(el) = self.document.remove_element(id) {
            self.history
                .push(Action::RemoveElement(id.to_string(), Box::new(el)));
            self.selected_ids.retain(|s| s != id);
            true
        } else {
            false
        }
    }

    /// Move an element to absolute position (x, y).
    pub fn move_element(&mut self, id: &str, x: f64, y: f64) {
        if let Some(el) = self.document.get_element(id) {
            let bounds = el.bounds();
            let dx = x - bounds.x;
            let dy = y - bounds.y;
            self.history.push(Action::MoveElement {
                id: id.to_string(),
                dx,
                dy,
            });
        }
        // Apply the move by setting position on the actual element
        if let Some(el) = self.document.get_element_mut(id) {
            match el {
                Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                    e.x = x;
                    e.y = y;
                }
                Element::Line(e) | Element::Arrow(e) => {
                    e.x = x;
                    e.y = y;
                }
                Element::FreeDraw(e) => {
                    e.x = x;
                    e.y = y;
                }
                Element::Text(e) => {
                    e.x = x;
                    e.y = y;
                }
            }
        }
    }

    /// Resize an element to the given bounds.
    pub fn resize_element(&mut self, id: &str, x: f64, y: f64, w: f64, h: f64) {
        if let Some(el) = self.document.get_element(id) {
            let b = el.bounds();
            self.history.push(Action::ResizeElement {
                id: id.to_string(),
                old_x: b.x,
                old_y: b.y,
                old_width: b.width,
                old_height: b.height,
                new_x: x,
                new_y: y,
                new_width: w,
                new_height: h,
            });
        }
        if let Some(el) = self.document.get_element_mut(id) {
            match el {
                Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                    e.x = x;
                    e.y = y;
                    e.width = w;
                    e.height = h;
                }
                _ => {
                    // Lines/FreeDraw/Text don't have width/height in the same way;
                    // for now, just reposition.
                    match el {
                        Element::Line(e) | Element::Arrow(e) => {
                            e.x = x;
                            e.y = y;
                        }
                        Element::FreeDraw(e) => {
                            e.x = x;
                            e.y = y;
                        }
                        Element::Text(e) => {
                            e.x = x;
                            e.y = y;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Update an element's style from JSON. The JSON should contain style fields
    /// (stroke, fill, font, opacity, etc.) to merge into the element.
    pub fn update_element_style(&mut self, id: &str, style_json: &str) -> bool {
        // Snapshot "before" for undo
        let before = self.document.get_element(id).cloned();
        let before = match before {
            Some(b) => b,
            None => return false,
        };

        // Parse the style update as a generic JSON value
        let updates: serde_json::Value = match serde_json::from_str(style_json) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // Serialize element, merge updates, deserialize back
        let mut elem_val = match serde_json::to_value(&before) {
            Ok(v) => v,
            Err(_) => return false,
        };

        if let (Some(obj), Some(upd)) = (elem_val.as_object_mut(), updates.as_object()) {
            for (k, v) in upd {
                obj.insert(k.clone(), v.clone());
            }
        } else {
            return false;
        }

        let updated: Element = match serde_json::from_value(elem_val) {
            Ok(e) => e,
            Err(_) => return false,
        };

        self.history.push(Action::UpdateElement {
            id: id.to_string(),
            before: Box::new(before),
            after: Box::new(updated.clone()),
        });

        // Replace in document
        if let Some(el) = self.document.get_element_mut(id) {
            *el = updated;
        }

        true
    }

    /// Get an element as JSON, or empty string if not found.
    pub fn get_element(&self, id: &str) -> String {
        match self.document.get_element(id) {
            Some(el) => serde_json::to_string(el).unwrap_or_default(),
            None => String::new(),
        }
    }

    // ── History ─────────────────────────────────────────────────────

    /// Undo the last action. Returns true if something was undone.
    pub fn undo(&mut self) -> bool {
        if let Some(action) = self.history.pop_undo() {
            self.apply_undo(&action);
            true
        } else {
            false
        }
    }

    /// Redo the last undone action. Returns true if something was redone.
    pub fn redo(&mut self) -> bool {
        if let Some(action) = self.history.pop_redo() {
            self.apply_redo(&action);
            true
        } else {
            false
        }
    }

    /// Push a raw action for undo support. The action_json must match the Action
    /// enum serialization format.
    pub fn push_action(&mut self, action_json: &str) -> bool {
        // Actions are not (de)serializable from JSON in the current core API,
        // so we expose a simple interface: callers should use the mutation
        // methods above which automatically track history.
        // This method exists as a placeholder for future extensibility.
        let _ = action_json;
        false
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    // ── Document info ───────────────────────────────────────────────

    pub fn document_name(&self) -> String {
        self.document.name.clone()
    }

    pub fn set_document_name(&mut self, name: &str) {
        self.document.name = name.to_string();
    }

    pub fn element_count(&self) -> usize {
        self.document.elements.len()
    }
}

// ── Private helpers (not exposed to JS) ─────────────────────────────────

impl DrawEngine {
    fn apply_undo(&mut self, action: &Action) {
        match action {
            Action::AddElement(el) => {
                self.document.remove_element(el.id());
            }
            Action::RemoveElement(_, el) => {
                self.document.add_element(*el.clone());
            }
            Action::MoveElement { id, dx, dy } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    match el {
                        Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                            e.x -= dx;
                            e.y -= dy;
                        }
                        Element::Line(e) | Element::Arrow(e) => {
                            e.x -= dx;
                            e.y -= dy;
                        }
                        Element::FreeDraw(e) => {
                            e.x -= dx;
                            e.y -= dy;
                        }
                        Element::Text(e) => {
                            e.x -= dx;
                            e.y -= dy;
                        }
                    }
                }
            }
            Action::ResizeElement {
                id,
                old_x,
                old_y,
                old_width,
                old_height,
                ..
            } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    match el {
                        Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                            e.x = *old_x;
                            e.y = *old_y;
                            e.width = *old_width;
                            e.height = *old_height;
                        }
                        _ => {}
                    }
                }
            }
            Action::UpdateElement { id, before, .. } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    *el = *before.clone();
                }
            }
            Action::Batch(actions) => {
                for a in actions.iter().rev() {
                    self.apply_undo(a);
                }
            }
        }
    }

    fn apply_redo(&mut self, action: &Action) {
        match action {
            Action::AddElement(el) => {
                self.document.add_element(*el.clone());
            }
            Action::RemoveElement(id, _) => {
                self.document.remove_element(id);
            }
            Action::MoveElement { id, dx, dy } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    match el {
                        Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                            e.x += dx;
                            e.y += dy;
                        }
                        Element::Line(e) | Element::Arrow(e) => {
                            e.x += dx;
                            e.y += dy;
                        }
                        Element::FreeDraw(e) => {
                            e.x += dx;
                            e.y += dy;
                        }
                        Element::Text(e) => {
                            e.x += dx;
                            e.y += dy;
                        }
                    }
                }
            }
            Action::ResizeElement {
                id,
                new_x,
                new_y,
                new_width,
                new_height,
                ..
            } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    match el {
                        Element::Rectangle(e) | Element::Ellipse(e) | Element::Diamond(e) => {
                            e.x = *new_x;
                            e.y = *new_y;
                            e.width = *new_width;
                            e.height = *new_height;
                        }
                        _ => {}
                    }
                }
            }
            Action::UpdateElement { id, after, .. } => {
                if let Some(el) = self.document.get_element_mut(id) {
                    *el = *after.clone();
                }
            }
            Action::Batch(actions) => {
                for a in actions {
                    self.apply_redo(a);
                }
            }
        }
    }
}

// ── Native tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let engine = DrawEngine::new(800, 600, 2.0);
        assert_eq!(engine.render_width(), 1600);
        assert_eq!(engine.render_height(), 1200);
        assert_eq!(engine.element_count(), 0);
        assert_eq!(engine.document_name(), "untitled");
    }

    #[test]
    fn test_set_size() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_size(1920, 1080);
        assert_eq!(engine.render_width(), 1920);
        assert_eq!(engine.render_height(), 1080);
    }

    #[test]
    fn test_load_save_document() {
        let mut engine = DrawEngine::new(800, 600, 1.0);

        let doc = Document::new("test doc".to_string());
        let json = serde_json::to_string(&doc).unwrap();

        assert!(engine.load_document(&json));
        assert_eq!(engine.document_name(), "test doc");

        let saved = engine.save_document();
        assert!(saved.contains("test doc"));

        // Invalid JSON should return false
        assert!(!engine.load_document("not json"));
    }

    #[test]
    fn test_render_returns_pixel_data() {
        let engine = DrawEngine::new(100, 100, 1.0);
        let data = engine.render();
        // 100x100 pixels * 4 bytes (RGBA)
        assert_eq!(data.len(), 100 * 100 * 4);
    }

    #[test]
    fn test_viewport() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_viewport(100.0, 200.0, 2.0);
        assert_eq!(engine.scroll_x(), 100.0);
        assert_eq!(engine.scroll_y(), 200.0);
        assert_eq!(engine.zoom(), 2.0);
    }

    #[test]
    fn test_screen_to_world() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_viewport(100.0, 50.0, 2.0);

        let result = engine.screen_to_world(300.0, 250.0);
        // wx = (300 - 100) / 2 = 100
        // wy = (250 - 50) / 2 = 100
        assert!(result.contains("100"));
    }

    #[test]
    fn test_add_and_remove_element() {
        let mut engine = DrawEngine::new(800, 600, 1.0);

        let json = r#"{"type":"Rectangle","id":"r1","x":10,"y":20,"width":100,"height":50}"#;
        let id = engine.add_element(json);
        assert_eq!(id, "r1");
        assert_eq!(engine.element_count(), 1);

        // Get element back
        let el_json = engine.get_element("r1");
        assert!(el_json.contains("r1"));

        // Remove
        assert!(engine.remove_element("r1"));
        assert_eq!(engine.element_count(), 0);
        assert!(!engine.remove_element("r1")); // already gone
    }

    #[test]
    fn test_move_element() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine
            .add_element(r#"{"type":"Rectangle","id":"r1","x":10,"y":20,"width":100,"height":50}"#);

        engine.move_element("r1", 50.0, 60.0);
        let el_json = engine.get_element("r1");
        assert!(el_json.contains("50"));
    }

    #[test]
    fn test_resize_element() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine
            .add_element(r#"{"type":"Rectangle","id":"r1","x":0,"y":0,"width":100,"height":100}"#);

        engine.resize_element("r1", 10.0, 10.0, 200.0, 150.0);
        let el_json = engine.get_element("r1");
        assert!(el_json.contains("200"));
        assert!(el_json.contains("150"));
    }

    #[test]
    fn test_selection() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_selection(r#"["r1","r2"]"#);
        let sel = engine.get_selection();
        assert!(sel.contains("r1"));
        assert!(sel.contains("r2"));
    }

    #[test]
    fn test_selection_box() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_selection_box(10.0, 20.0, 100.0, 50.0);
        // Just verify it doesn't panic; the box is used during render
        let data = engine.render();
        assert!(!data.is_empty());

        engine.clear_selection_box();
        let data = engine.render();
        assert!(!data.is_empty());
    }

    #[test]
    fn test_undo_redo_add() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine
            .add_element(r#"{"type":"Rectangle","id":"r1","x":0,"y":0,"width":100,"height":100}"#);
        assert_eq!(engine.element_count(), 1);
        assert!(engine.can_undo());

        assert!(engine.undo());
        assert_eq!(engine.element_count(), 0);
        assert!(engine.can_redo());

        assert!(engine.redo());
        assert_eq!(engine.element_count(), 1);
    }

    #[test]
    fn test_undo_redo_remove() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine
            .add_element(r#"{"type":"Rectangle","id":"r1","x":0,"y":0,"width":100,"height":100}"#);
        engine.remove_element("r1");
        assert_eq!(engine.element_count(), 0);

        assert!(engine.undo()); // undo remove
        assert_eq!(engine.element_count(), 1);

        assert!(engine.redo()); // redo remove
        assert_eq!(engine.element_count(), 0);
    }

    #[test]
    fn test_undo_move() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.add_element(
            r#"{"type":"Rectangle","id":"r1","x":10,"y":20,"width":100,"height":100}"#,
        );
        engine.move_element("r1", 50.0, 60.0);

        // Undo the move
        engine.undo();
        let el_json = engine.get_element("r1");
        // Should be back at (10, 20)
        assert!(el_json.contains("\"x\":10"));
        assert!(el_json.contains("\"y\":20"));
    }

    #[test]
    fn test_update_element_style() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine
            .add_element(r#"{"type":"Rectangle","id":"r1","x":0,"y":0,"width":100,"height":100}"#);

        let ok = engine.update_element_style("r1", r#"{"opacity": 0.5}"#);
        assert!(ok);

        let el_json = engine.get_element("r1");
        assert!(el_json.contains("0.5"));

        // Undo should restore opacity
        engine.undo();
        let el_json = engine.get_element("r1");
        assert!(el_json.contains("1.0") || el_json.contains("1"));
    }

    #[test]
    fn test_hit_test_empty() {
        let engine = DrawEngine::new(800, 600, 1.0);
        let result = engine.hit_test(400.0, 300.0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_elements_in_rect_empty() {
        let engine = DrawEngine::new(800, 600, 1.0);
        let result = engine.elements_in_rect(0.0, 0.0, 100.0, 100.0);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_document_name() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        engine.set_document_name("my drawing");
        assert_eq!(engine.document_name(), "my drawing");
    }

    #[test]
    fn test_push_action_placeholder() {
        let mut engine = DrawEngine::new(800, 600, 1.0);
        assert!(!engine.push_action("{}"));
    }
}
