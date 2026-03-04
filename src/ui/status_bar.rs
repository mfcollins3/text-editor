use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

/// The bottom status bar showing filename, modified indicator, and cursor
/// position.
pub struct StatusBar {
    pub file_name: String,
    pub is_dirty: bool,
    pub line: usize,
    pub col: usize,
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let modified = if self.is_dirty { " [+]" } else { "" };
        let left = format!(" {}{}", self.file_name, modified);
        let right = format!("Ln {}, Col {} ", self.line, self.col);

        let padding = area.width as usize
            - left.len().min(area.width as usize)
            - right.len().min(area.width as usize);
        let middle = " ".repeat(padding.min(area.width as usize));

        let style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let line = Line::from(vec![
            Span::styled(left, style),
            Span::styled(middle, style),
            Span::styled(right, style),
        ]);

        Paragraph::new(line).render(area, buf);
    }
}
