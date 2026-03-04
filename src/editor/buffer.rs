use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

use ropey::Rope;

/// Rope-based text buffer that tracks dirty state and file path.
pub struct Buffer {
    rope: Rope,
    file_path: Option<PathBuf>,
    is_dirty: bool,
    saved_hash: u64,
}

impl Buffer {
    /// Create an empty buffer.
    pub fn new() -> Self {
        let rope = Rope::new();
        let hash = hash_rope(&rope);
        Self {
            rope,
            file_path: None,
            is_dirty: false,
            saved_hash: hash,
        }
    }

    /// Load a buffer from a file on disk.
    pub fn from_file(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let text = std::fs::read_to_string(&path)?;
        let rope = Rope::from_str(&text);
        let hash = hash_rope(&rope);
        Ok(Self {
            rope,
            file_path: Some(path),
            is_dirty: false,
            saved_hash: hash,
        })
    }

    /// Initialise a buffer from a template string.
    pub fn from_template(template: impl Into<String>) -> Self {
        let rope = Rope::from_str(&template.into());
        let hash = hash_rope(&rope);
        Self {
            rope,
            file_path: None,
            is_dirty: true, // template content counts as unsaved
            saved_hash: hash,
        }
    }

    /// Insert a single character at the given rope character index.
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        self.rope.insert_char(char_idx, ch);
        self.mark_dirty();
    }

    /// Insert a string at the given rope character index.
    pub fn insert_text(&mut self, char_idx: usize, text: &str) {
        self.rope.insert(char_idx, text);
        self.mark_dirty();
    }

    /// Delete characters in `[start, end)` (rope character indices).
    pub fn delete_range(&mut self, start: usize, end: usize) {
        self.rope.remove(start..end);
        self.mark_dirty();
    }

    /// Return the full document as a `String`.
    pub fn get_text(&self) -> String {
        self.rope.to_string()
    }

    /// Replace the entire document with `text`.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.rope = Rope::from_str(&text.into());
        self.mark_dirty();
    }

    /// Save the buffer to its associated file path.
    pub fn save(&mut self) -> io::Result<()> {
        let path = self
            .file_path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no file path set"))?;
        let text = self.rope.to_string();
        std::fs::write(path, &text)?;
        self.saved_hash = hash_rope(&self.rope);
        self.is_dirty = false;
        Ok(())
    }

    /// Set (or update) the associated file path without saving.
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.file_path = Some(path.into());
    }

    /// Return the associated file path, if any.
    pub fn file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    /// Return `true` if the buffer has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Return the number of lines in the rope.
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Return the length of the buffer in characters.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Return a slice of the rope as a `String` for the given logical line.
    pub fn line(&self, line_idx: usize) -> String {
        self.rope.line(line_idx).to_string()
    }

    /// Convert a `(line, col)` pair to a rope character index.
    pub fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_char(line);
        line_start + col
    }

    /// Convert a rope character index to a `(line, col)` pair.
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let line = self.rope.char_to_line(char_idx);
        let line_start = self.rope.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }

    /// Called on every application tick (used by recovery).
    pub fn tick(&self) {
        // Periodic tasks are driven by RecoveryManager; nothing to do here yet.
    }

    // --- private helpers ---

    fn mark_dirty(&mut self) {
        let current = hash_rope(&self.rope);
        self.is_dirty = current != self.saved_hash;
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_rope(rope: &Rope) -> u64 {
    let mut hasher = DefaultHasher::new();
    rope.to_string().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_buffer_is_not_dirty() {
        let buf = Buffer::new();
        assert!(!buf.is_dirty());
    }

    #[test]
    fn insert_char_marks_dirty() {
        let mut buf = Buffer::new();
        buf.insert_char(0, 'H');
        assert!(buf.is_dirty());
        assert_eq!(buf.get_text(), "H");
    }

    #[test]
    fn set_text_and_get_text_roundtrip() {
        let mut buf = Buffer::new();
        buf.set_text("hello");
        assert_eq!(buf.get_text(), "hello");
    }

    #[test]
    fn delete_range_works() {
        let mut buf = Buffer::new();
        buf.set_text("hello world");
        buf.delete_range(5, 11);
        assert_eq!(buf.get_text(), "hello");
    }

    #[test]
    fn from_template_is_dirty() {
        let buf = Buffer::from_template("# Title\n");
        assert!(buf.is_dirty());
    }
}
