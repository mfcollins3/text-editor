use crate::editor::cursor::Position;
use crate::editor::render::MarkdownEditorState;

/// Every operation that can be performed on the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorAction {
    // --- Cursor movement ---
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordLeft,
    MoveWordRight,
    MoveLineStart,
    MoveLineEnd,
    MoveDocumentStart,
    MoveDocumentEnd,

    // --- Selection ---
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectWordLeft,
    SelectWordRight,
    SelectLineStart,
    SelectLineEnd,
    SelectDocumentStart,
    SelectDocumentEnd,
    SelectAll,
    ClearSelection,

    // --- Editing ---
    InsertChar(char),
    InsertText(String),
    InsertNewline,
    DeleteBackward,
    DeleteForward,
    DeleteWordBackward,
    DeleteWordForward,
    Indent,
    Dedent,

    // --- Clipboard ---
    Copy,
    Cut,
    Paste,

    // --- History ---
    Undo,
    Redo,

    // --- Markdown shortcuts ---
    ToggleBold,
    ToggleItalic,
    ToggleInlineCode,
    InsertLink,

    // --- Overlays ---
    OpenSlashCommand,
    OpenMention,
    CloseOverlay,

    // --- File operations ---
    Save,
    Quit,

    // --- View toggles ---
    TogglePreview,
    ToggleLineNumbers,
}

/// Dispatch an action against the current editor state and return the
/// corresponding `EditorResult`.
pub fn dispatch(
    action: EditorAction,
    state: &mut MarkdownEditorState,
) -> crate::editor::EditorResult {
    use crate::editor::EditorResult;
    use EditorAction::*;

    match action {
        InsertChar(ch) => {
            let char_idx = state
                .buffer
                .line_col_to_char(state.cursor.pos.line, state.cursor.pos.col);
            state.buffer.insert_char(char_idx, ch);
            // Advance cursor
            state.cursor.pos.col += 1;
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        InsertNewline => {
            let char_idx = state
                .buffer
                .line_col_to_char(state.cursor.pos.line, state.cursor.pos.col);
            state.buffer.insert_char(char_idx, '\n');
            state.cursor.pos.line += 1;
            state.cursor.pos.col = 0;
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        DeleteBackward => {
            if state.cursor.pos.col > 0 || state.cursor.pos.line > 0 {
                let char_idx = state
                    .buffer
                    .line_col_to_char(state.cursor.pos.line, state.cursor.pos.col);
                if char_idx > 0 {
                    state.buffer.delete_range(char_idx - 1, char_idx);
                    let (line, col) = state.buffer.char_to_line_col(char_idx - 1);
                    state.cursor.pos = Position::new(line, col);
                    state.cursor.clear_selection();
                }
            }
            EditorResult::Consumed
        }

        DeleteForward => {
            let char_idx = state
                .buffer
                .line_col_to_char(state.cursor.pos.line, state.cursor.pos.col);
            if char_idx < state.buffer.len_chars() {
                state.buffer.delete_range(char_idx, char_idx + 1);
                state.cursor.clear_selection();
            }
            EditorResult::Consumed
        }

        MoveLeft => {
            if state.cursor.pos.col > 0 {
                state.cursor.pos.col -= 1;
            } else if state.cursor.pos.line > 0 {
                let prev_line = state.cursor.pos.line - 1;
                let line_text = state.buffer.line(prev_line);
                let col = line_text.trim_end_matches('\n').chars().count();
                state.cursor.pos = Position::new(prev_line, col);
            }
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveRight => {
            let line_text = state.buffer.line(state.cursor.pos.line);
            let line_len = line_text.trim_end_matches('\n').chars().count();
            if state.cursor.pos.col < line_len {
                state.cursor.pos.col += 1;
            } else if state.cursor.pos.line + 1 < state.buffer.len_lines() {
                state.cursor.pos = Position::new(state.cursor.pos.line + 1, 0);
            }
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveUp => {
            if state.cursor.pos.line > 0 {
                let preferred = state.cursor.preferred_col;
                let prev_line = state.cursor.pos.line - 1;
                let line_text = state.buffer.line(prev_line);
                let line_len = line_text.trim_end_matches('\n').chars().count();
                state.cursor.pos = Position::new(prev_line, preferred.min(line_len));
            }
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveDown => {
            let preferred = state.cursor.preferred_col;
            let next_line = state.cursor.pos.line + 1;
            if next_line < state.buffer.len_lines() {
                let line_text = state.buffer.line(next_line);
                let line_len = line_text.trim_end_matches('\n').chars().count();
                state.cursor.pos = Position::new(next_line, preferred.min(line_len));
            }
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveLineStart => {
            state.cursor.pos.col = 0;
            state.cursor.preferred_col = 0;
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveLineEnd => {
            let line_text = state.buffer.line(state.cursor.pos.line);
            let col = line_text.trim_end_matches('\n').chars().count();
            state.cursor.pos.col = col;
            state.cursor.preferred_col = col;
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        MoveDocumentStart => {
            state.cursor.move_to(Position::default());
            EditorResult::Consumed
        }

        MoveDocumentEnd => {
            let last_line = state.buffer.len_lines().saturating_sub(1);
            let line_text = state.buffer.line(last_line);
            let col = line_text.trim_end_matches('\n').chars().count();
            state.cursor.move_to(Position::new(last_line, col));
            EditorResult::Consumed
        }

        SelectLeft => {
            state.cursor.start_selection();
            dispatch(MoveLeft, state);
            EditorResult::Consumed
        }

        SelectRight => {
            state.cursor.start_selection();
            dispatch(MoveRight, state);
            EditorResult::Consumed
        }

        SelectUp => {
            state.cursor.start_selection();
            dispatch(MoveUp, state);
            EditorResult::Consumed
        }

        SelectDown => {
            state.cursor.start_selection();
            dispatch(MoveDown, state);
            EditorResult::Consumed
        }

        SelectAll => {
            state.cursor.pos = Position::default();
            state.cursor.start_selection();
            let last_line = state.buffer.len_lines().saturating_sub(1);
            let line_text = state.buffer.line(last_line);
            let col = line_text.trim_end_matches('\n').chars().count();
            state.cursor.pos = Position::new(last_line, col);
            EditorResult::Consumed
        }

        ClearSelection => {
            state.cursor.clear_selection();
            EditorResult::Consumed
        }

        TogglePreview => {
            state.config.show_preview = !state.config.show_preview;
            EditorResult::Consumed
        }

        ToggleLineNumbers => {
            state.config.show_line_numbers = !state.config.show_line_numbers;
            EditorResult::Consumed
        }

        Save => EditorResult::Save,
        Quit => EditorResult::Quit,

        // Remaining actions are stubs for future phases.
        _ => EditorResult::Ignored,
    }
}
