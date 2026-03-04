/// Scroll position and visible region tracking.
#[derive(Debug, Clone, Default)]
pub struct Viewport {
    /// First visible logical line (0-indexed).
    pub scroll_top: usize,
    /// Number of visible rows available for the editor area.
    pub height: usize,
    /// Number of visible columns available for the editor area.
    pub width: usize,
}

impl Viewport {
    /// Scroll so that `line` is visible.
    pub fn ensure_visible(&mut self, line: usize) {
        if line < self.scroll_top {
            self.scroll_top = line;
        } else if self.height > 0 && line >= self.scroll_top + self.height {
            self.scroll_top = line - self.height + 1;
        }
    }

    /// Scroll up by `n` lines.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_top = self.scroll_top.saturating_sub(n);
    }

    /// Scroll down by `n` lines.
    pub fn scroll_down(&mut self, n: usize, total_lines: usize) {
        let max = total_lines.saturating_sub(1);
        self.scroll_top = (self.scroll_top + n).min(max);
    }

    /// Return the range of logical lines that are currently visible.
    pub fn visible_range(&self) -> std::ops::Range<usize> {
        self.scroll_top..self.scroll_top + self.height
    }
}
