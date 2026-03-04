use ratatui::buffer::Buffer as RatatuiBuffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget};

use crate::editor::buffer::Buffer;
use crate::editor::cursor::Cursor;
use crate::editor::highlight::{highlight_buffer, MarkdownTheme};
use crate::editor::history::History;
use crate::editor::overlay::OverlayManager;
use crate::editor::preview::render_preview;
use crate::editor::viewport::Viewport;
use crate::editor::wrap::wrap_line;
use crate::ui::help_bar::HelpBar;
use crate::ui::status_bar::StatusBar;

/// Runtime configuration for the editor.
#[derive(Debug, Clone)]
pub struct EditorConfig {
    /// Number of spaces per tab stop.
    pub tab_width: usize,
    /// Show the bottom status bar.
    pub show_status_bar: bool,
    /// Show the top/bottom help hints bar.
    pub show_help_hints: bool,
    /// Show line numbers in the gutter.
    pub show_line_numbers: bool,
    /// Show the Markdown preview pane.
    pub show_preview: bool,
    /// Syntax highlight theme.
    pub theme: MarkdownTheme,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            tab_width: 4,
            show_status_bar: true,
            show_help_hints: true,
            show_line_numbers: false,
            show_preview: false,
            theme: MarkdownTheme::default(),
        }
    }
}

/// The full state managed by the host application and passed to the widget.
pub struct MarkdownEditorState {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub history: History,
    pub viewport: Viewport,
    pub highlight_cache: Vec<Vec<crate::editor::highlight::StyledRange>>,
    pub overlay: OverlayManager,
    pub config: EditorConfig,
}

/// The stateless widget that renders a `MarkdownEditorState`.
pub struct MarkdownEditorWidget;

impl StatefulWidget for MarkdownEditorWidget {
    type State = MarkdownEditorState;

    fn render(self, area: Rect, buf: &mut RatatuiBuffer, state: &mut Self::State) {
        // Reserve rows for status bar and help hints.
        let status_height: u16 = if state.config.show_status_bar { 1 } else { 0 };
        let help_height: u16 = if state.config.show_help_hints { 1 } else { 0 };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(help_height),
                Constraint::Min(1),
                Constraint::Length(status_height),
            ])
            .split(area);

        let help_area = rows[0];
        let editor_area = rows[1];
        let status_area = rows[2];

        // Render help bar.
        if state.config.show_help_hints {
            HelpBar.render(help_area, buf);
        }

        // Render status bar.
        if state.config.show_status_bar {
            let file_name = state
                .buffer
                .file_path()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("[No Name]")
                .to_string();
            StatusBar {
                file_name,
                is_dirty: state.buffer.is_dirty(),
                line: state.cursor.pos.line + 1,
                col: state.cursor.pos.col + 1,
            }
            .render(status_area, buf);
        }

        // Split editor area for optional preview pane.
        let (edit_area, preview_area) = if state.config.show_preview {
            let halves = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(editor_area);
            (halves[0], Some(halves[1]))
        } else {
            (editor_area, None)
        };

        // Update viewport dimensions.
        state.viewport.height = edit_area.height as usize;
        state.viewport.width = edit_area.width as usize;
        state.viewport.ensure_visible(state.cursor.pos.line);

        // Rebuild highlight cache if it's stale.
        let text = state.buffer.get_text();
        if state.highlight_cache.len() != state.buffer.len_lines() {
            state.highlight_cache = highlight_buffer(&text, &state.config.theme);
        }

        // Gutter width (for line numbers).
        let gutter_width: u16 = if state.config.show_line_numbers {
            let total = state.buffer.len_lines();
            let digits = total.to_string().len() as u16;
            digits + 1 // one space padding
        } else {
            0
        };

        let text_x = edit_area.x + gutter_width;
        let text_width = edit_area.width.saturating_sub(gutter_width) as usize;

        // Render each visible row.
        let scroll_top = state.viewport.scroll_top;
        let visible_height = edit_area.height as usize;

        let cursor_line = state.cursor.pos.line;
        let cursor_col = state.cursor.pos.col;

        let lines: Vec<&str> = text.lines().collect();

        let mut screen_row = 0usize;
        let mut logical_line = scroll_top;

        while screen_row < visible_height && logical_line < lines.len() {
            let line_text = lines[logical_line];
            let visual_lines = wrap_line(logical_line, line_text, text_width.max(1));

            for vl in &visual_lines {
                if screen_row >= visible_height {
                    break;
                }

                let y = edit_area.y + screen_row as u16;

                // Draw line number gutter.
                if state.config.show_line_numbers && vl.is_first {
                    let gutter_str = format!("{:>number_width$} ", logical_line + 1, number_width = gutter_width as usize - 1);
                    let gutter_line = Line::styled(gutter_str, Style::default().fg(Color::DarkGray));
                    let gutter_area = Rect {
                        x: edit_area.x,
                        y,
                        width: gutter_width,
                        height: 1,
                    };
                    Paragraph::new(gutter_line).render(gutter_area, buf);
                }

                // Build the text for this visual line.
                let segment: String = line_text
                    .chars()
                    .skip(vl.start_col)
                    .take(vl.end_col - vl.start_col)
                    .collect();

                let line_widget = Line::raw(segment);
                let text_area = Rect {
                    x: text_x,
                    y,
                    width: edit_area.width.saturating_sub(gutter_width),
                    height: 1,
                };
                Paragraph::new(line_widget).render(text_area, buf);

                screen_row += 1;
            }

            logical_line += 1;
        }

        // Render cursor (simple block cursor).
        {
            let cursor_screen_row = cursor_line.saturating_sub(scroll_top);
            if cursor_screen_row < visible_height {
                let line_text = lines.get(cursor_line).copied().unwrap_or("");
                let visual_lines = wrap_line(cursor_line, line_text, text_width.max(1));

                // Find which visual row contains the cursor column.
                let mut visual_col = cursor_col;
                for vl in &visual_lines {
                    if cursor_col >= vl.start_col && cursor_col <= vl.end_col {
                        visual_col = cursor_col - vl.start_col;
                        break;
                    }
                }

                let cursor_x = text_x + visual_col as u16;
                let cursor_y = edit_area.y + cursor_screen_row as u16;

                if cursor_x < edit_area.x + edit_area.width && cursor_y < edit_area.y + edit_area.height {
                    // Invert the cell under the cursor.
                    let cell = buf.cell_mut((cursor_x, cursor_y));
                    if let Some(cell) = cell {
                        let style = cell.style().patch(Style::default().bg(Color::White).fg(Color::Black));
                        cell.set_style(style);
                    }
                }
            }
        }

        // Render preview pane if active.
        if let Some(preview_area) = preview_area {
            let preview_lines = render_preview(&text);
            let preview = Paragraph::new(preview_lines)
                .block(Block::bordered().title("Preview"));
            preview.render(preview_area, buf);
        }

        // Render any active overlay.
        state.overlay.render(edit_area, buf);
    }
}
