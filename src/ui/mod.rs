pub mod help_bar;
pub mod status_bar;

use ratatui::Frame;

use crate::editor::render::MarkdownEditorWidget;
use crate::editor::MarkdownEditor;

/// Draw the entire UI for a `MarkdownEditor` into `frame`.
pub fn draw(frame: &mut Frame, editor: &mut MarkdownEditor) {
    let area = frame.area();
    frame.render_stateful_widget(MarkdownEditorWidget, area, editor.state_mut());
}
