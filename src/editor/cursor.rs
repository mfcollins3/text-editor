/// A character-based position within the document (0-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// Logical (unwrapped) line index.
    pub line: usize,
    /// Character column within the logical line.
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line.cmp(&other.line).then(self.col.cmp(&other.col))
    }
}

/// Cursor state: current position, optional selection anchor, and preferred
/// column for vertical movement.
#[derive(Debug, Clone, Default)]
pub struct Cursor {
    /// The active cursor position (insertion point).
    pub pos: Position,
    /// The fixed end of a selection; `None` when there is no selection.
    pub anchor: Option<Position>,
    /// Preferred column for up/down movement (sticky column).
    pub preferred_col: usize,
}

impl Cursor {
    /// Move the cursor to `pos`, clearing any selection.
    pub fn move_to(&mut self, pos: Position) {
        self.pos = pos;
        self.anchor = None;
        self.preferred_col = pos.col;
    }

    /// Begin or extend a selection.  The anchor is set to the current position
    /// before the first call; subsequent calls only update `pos`.
    pub fn start_selection(&mut self) {
        if self.anchor.is_none() {
            self.anchor = Some(self.pos);
        }
    }

    /// Clear the selection without moving the cursor.
    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// Return `true` if a selection is active.
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }

    /// Return the ordered `(start, end)` range of the current selection.
    /// Returns `None` when there is no selection.
    pub fn selection_range(&self) -> Option<(Position, Position)> {
        self.anchor.map(|anchor| {
            if anchor <= self.pos {
                (anchor, self.pos)
            } else {
                (self.pos, anchor)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_ordering() {
        let a = Position::new(0, 5);
        let b = Position::new(1, 0);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, a);
    }

    #[test]
    fn selection_range_ordered() {
        let mut cursor = Cursor::default();
        cursor.pos = Position::new(2, 10);
        cursor.anchor = Some(Position::new(1, 3));
        let (start, end) = cursor.selection_range().unwrap();
        assert!(start <= end);
    }

    #[test]
    fn clear_selection() {
        let mut cursor = Cursor::default();
        cursor.start_selection();
        cursor.pos = Position::new(0, 5);
        assert!(cursor.has_selection());
        cursor.clear_selection();
        assert!(!cursor.has_selection());
    }
}
