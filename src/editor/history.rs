use std::time::{Duration, Instant};

use crate::editor::cursor::Position;

/// The coalescing window for grouping rapid edits into a single undo step.
const GROUP_TIMEOUT: Duration = Duration::from_millis(300);

/// The kind of change recorded in an edit entry.
#[derive(Debug, Clone)]
pub enum EditKind {
    Insert,
    Delete,
    Replace,
}

/// A single recorded change to the buffer.
#[derive(Debug, Clone)]
pub struct Edit {
    pub kind: EditKind,
    /// Text that was in place before the edit (used for undo).
    pub old_text: String,
    /// Text after the edit (used for redo).
    pub new_text: String,
    /// Character index in the rope where the change started.
    pub char_idx: usize,
    pub cursor_before: Position,
    pub cursor_after: Position,
    pub timestamp: Instant,
}

/// A group of edits that should be undone/redone together.
#[derive(Debug, Clone)]
pub struct EditGroup {
    pub edits: Vec<Edit>,
    /// Timestamp of the last edit appended to this group.
    pub last_edit_time: Instant,
}

impl EditGroup {
    fn new(edit: Edit) -> Self {
        let ts = edit.timestamp;
        Self {
            edits: vec![edit],
            last_edit_time: ts,
        }
    }

    /// Return `true` if `edit` is close enough in time to be coalesced.
    fn can_coalesce(&self, edit: &Edit) -> bool {
        edit.timestamp.duration_since(self.last_edit_time) <= GROUP_TIMEOUT
    }

    fn push(&mut self, edit: Edit) {
        self.last_edit_time = edit.timestamp;
        self.edits.push(edit);
    }
}

/// Unlimited undo/redo history.
pub struct History {
    undo_stack: Vec<EditGroup>,
    redo_stack: Vec<EditGroup>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Record an edit.  Rapid consecutive edits are coalesced.
    /// Any recorded redo state is discarded when a new edit arrives.
    pub fn push(&mut self, edit: Edit) {
        self.redo_stack.clear();

        if let Some(group) = self.undo_stack.last_mut() {
            if group.can_coalesce(&edit) {
                group.push(edit);
                return;
            }
        }
        self.undo_stack.push(EditGroup::new(edit));
    }

    /// Pop the most recent edit group for undoing.
    pub fn pop_undo(&mut self) -> Option<EditGroup> {
        let group = self.undo_stack.pop()?;
        self.redo_stack.push(group.clone());
        Some(group)
    }

    /// Pop the most recently undone edit group for redoing.
    pub fn pop_redo(&mut self) -> Option<EditGroup> {
        let group = self.redo_stack.pop()?;
        self.undo_stack.push(group.clone());
        Some(group)
    }

    /// Return `true` if there are changes that can be undone.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Return `true` if there are changes that can be redone.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edit(cursor: Position) -> Edit {
        Edit {
            kind: EditKind::Insert,
            old_text: String::new(),
            new_text: "a".to_string(),
            char_idx: 0,
            cursor_before: cursor,
            cursor_after: cursor,
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn push_and_undo() {
        let mut history = History::new();
        history.push(make_edit(Position::default()));
        assert!(history.can_undo());
        assert!(!history.can_redo());
        history.pop_undo();
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn undo_clears_redo_on_new_edit() {
        let mut history = History::new();
        history.push(make_edit(Position::default()));
        history.pop_undo();
        assert!(history.can_redo());
        history.push(make_edit(Position::default()));
        assert!(!history.can_redo());
    }
}
