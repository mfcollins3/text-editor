use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

const HINTS: &[(&str, &str)] = &[
    ("^S", "Save"),
    ("^Q", "Quit"),
    ("^B", "Bold"),
    ("^I", "Italic"),
    ("^`", "Code"),
    ("^K", "Link"),
    ("^P", "Preview"),
    ("/", "Cmds"),
    ("@", "Mention"),
];

/// A single-row help hints bar showing common key bindings.
pub struct HelpBar;

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let key_style = Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let label_style = Style::default().bg(Color::Black).fg(Color::White);

        let mut spans: Vec<Span> = Vec::new();
        for (key, label) in HINTS {
            spans.push(Span::styled(format!(" {key} "), key_style));
            spans.push(Span::styled(format!("{label} "), label_style));
        }

        Paragraph::new(Line::from(spans)).render(area, buf);
    }
}
