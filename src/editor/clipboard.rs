/// System clipboard integration via the `arboard` crate.
///
/// The clipboard is accessed lazily so that headless / test environments do not
/// fail at startup.
pub struct Clipboard {
    inner: Option<arboard::Clipboard>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Return the current clipboard text, or `None` on error.
    pub fn get_text(&mut self) -> Option<String> {
        self.board()?.get_text().ok()
    }

    /// Set the clipboard text.  Errors are silently ignored.
    pub fn set_text(&mut self, text: impl Into<String>) {
        if let Some(board) = self.board() {
            let _ = board.set_text(text.into());
        }
    }

    /// Lazily initialise (or return) the underlying `arboard::Clipboard`.
    fn board(&mut self) -> Option<&mut arboard::Clipboard> {
        if self.inner.is_none() {
            self.inner = arboard::Clipboard::new().ok();
        }
        self.inner.as_mut()
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}
