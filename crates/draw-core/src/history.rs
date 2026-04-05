use crate::element::Element;

#[derive(Debug, Clone)]
pub enum Action {
    AddElement(Box<Element>),
    RemoveElement(String, Box<Element>),
    MoveElement {
        id: String,
        dx: f64,
        dy: f64,
    },
    ResizeElement {
        id: String,
        old_x: f64,
        old_y: f64,
        old_width: f64,
        old_height: f64,
        new_x: f64,
        new_y: f64,
        new_width: f64,
        new_height: f64,
    },
    UpdateElement {
        id: String,
        before: Box<Element>,
        after: Box<Element>,
    },
    Batch(Vec<Action>),
}

#[derive(Debug, Default)]
pub struct History {
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn pop_undo(&mut self) -> Option<Action> {
        let action = self.undo_stack.pop()?;
        self.redo_stack.push(action.clone());
        Some(action)
    }

    pub fn pop_redo(&mut self) -> Option<Action> {
        let action = self.redo_stack.pop()?;
        self.undo_stack.push(action.clone());
        Some(action)
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::element::{Element, ShapeElement};
    use crate::style::{FillStyle, StrokeStyle};

    fn test_element(id: &str) -> Element {
        Element::Rectangle(ShapeElement {
            id: id.to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            angle: 0.0,
            stroke: StrokeStyle::default(),
            fill: FillStyle::default(),
            opacity: 1.0,
            locked: false,
            group_id: None,
        })
    }

    #[test]
    fn test_batch_counts_as_single_undo() {
        let mut history = History::new();
        let batch = Action::Batch(vec![
            Action::AddElement(Box::new(test_element("a"))),
            Action::AddElement(Box::new(test_element("b"))),
            Action::AddElement(Box::new(test_element("c"))),
        ]);
        history.push(batch);

        assert!(history.can_undo());
        assert!(!history.can_redo());

        // One pop_undo should return the entire batch
        let action = history.pop_undo().unwrap();
        assert!(
            matches!(&action, Action::Batch(actions) if actions.len() == 3),
            "expected Batch with 3 actions, got {action:?}"
        );

        // After one undo, stack should be empty
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_batch_redo_returns_batch() {
        let mut history = History::new();
        let batch = Action::Batch(vec![
            Action::RemoveElement("a".to_string(), Box::new(test_element("a"))),
            Action::RemoveElement("b".to_string(), Box::new(test_element("b"))),
        ]);
        history.push(batch);
        history.pop_undo();

        let action = history.pop_redo().unwrap();
        assert!(
            matches!(&action, Action::Batch(actions) if actions.len() == 2),
            "expected Batch with 2 actions, got {action:?}"
        );

        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut history = History::new();
        history.push(Action::AddElement(Box::new(test_element("a"))));
        history.pop_undo();
        assert!(history.can_redo());

        // Pushing a new batch should clear redo
        history.push(Action::Batch(vec![Action::AddElement(Box::new(
            test_element("b"),
        ))]));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_empty_batch() {
        let mut history = History::new();
        history.push(Action::Batch(vec![]));
        assert!(history.can_undo());

        let action = history.pop_undo().unwrap();
        assert!(
            matches!(&action, Action::Batch(actions) if actions.is_empty()),
            "expected empty Batch, got {action:?}"
        );
    }
}
