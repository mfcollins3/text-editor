use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::editor::actions::EditorAction;

/// Map a raw crossterm `KeyEvent` to an `EditorAction`.
/// Returns `None` if the key has no binding.
pub fn map_key_event(key: KeyEvent) -> Option<EditorAction> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        // --- Cursor movement ---
        KeyCode::Left if ctrl && shift => Some(EditorAction::SelectWordLeft),
        KeyCode::Right if ctrl && shift => Some(EditorAction::SelectWordRight),
        KeyCode::Left if ctrl => Some(EditorAction::MoveWordLeft),
        KeyCode::Right if ctrl => Some(EditorAction::MoveWordRight),
        KeyCode::Left if shift => Some(EditorAction::SelectLeft),
        KeyCode::Right if shift => Some(EditorAction::SelectRight),
        KeyCode::Up if shift => Some(EditorAction::SelectUp),
        KeyCode::Down if shift => Some(EditorAction::SelectDown),
        KeyCode::Left => Some(EditorAction::MoveLeft),
        KeyCode::Right => Some(EditorAction::MoveRight),
        KeyCode::Up => Some(EditorAction::MoveUp),
        KeyCode::Down => Some(EditorAction::MoveDown),

        // --- Line navigation ---
        KeyCode::Home if ctrl && shift => Some(EditorAction::SelectDocumentStart),
        KeyCode::End if ctrl && shift => Some(EditorAction::SelectDocumentEnd),
        KeyCode::Home if ctrl => Some(EditorAction::MoveDocumentStart),
        KeyCode::End if ctrl => Some(EditorAction::MoveDocumentEnd),
        KeyCode::Home if shift => Some(EditorAction::SelectLineStart),
        KeyCode::End if shift => Some(EditorAction::SelectLineEnd),
        KeyCode::Home => Some(EditorAction::MoveLineStart),
        KeyCode::End => Some(EditorAction::MoveLineEnd),

        // --- Editing ---
        KeyCode::Enter => Some(EditorAction::InsertNewline),
        KeyCode::Backspace => Some(EditorAction::DeleteBackward),
        KeyCode::Delete => Some(EditorAction::DeleteForward),
        KeyCode::Tab if shift => Some(EditorAction::Dedent),
        KeyCode::Tab => Some(EditorAction::Indent),

        // --- Ctrl shortcuts ---
        KeyCode::Char('a') if ctrl => Some(EditorAction::SelectAll),
        KeyCode::Char('c') if ctrl => Some(EditorAction::Copy),
        KeyCode::Char('x') if ctrl => Some(EditorAction::Cut),
        KeyCode::Char('v') if ctrl => Some(EditorAction::Paste),
        KeyCode::Char('z') if ctrl => Some(EditorAction::Undo),
        KeyCode::Char('y') if ctrl => Some(EditorAction::Redo),
        KeyCode::Char('s') if ctrl => Some(EditorAction::Save),
        KeyCode::Char('q') if ctrl => Some(EditorAction::Quit),
        KeyCode::Char('b') if ctrl => Some(EditorAction::ToggleBold),
        KeyCode::Char('i') if ctrl => Some(EditorAction::ToggleItalic),
        KeyCode::Char('k') if ctrl => Some(EditorAction::InsertLink),
        KeyCode::Char('p') if ctrl => Some(EditorAction::TogglePreview),
        KeyCode::Char('l') if ctrl => Some(EditorAction::ToggleLineNumbers),
        KeyCode::Char('`') if ctrl => Some(EditorAction::ToggleInlineCode),

        // --- Regular character input ---
        KeyCode::Char('/') if !ctrl && !shift => Some(EditorAction::InsertChar('/')),
        KeyCode::Char('@') if !ctrl => Some(EditorAction::InsertChar('@')),
        KeyCode::Char(ch) if !ctrl => Some(EditorAction::InsertChar(ch)),

        _ => None,
    }
}

/// Process typed `/` or `@` triggers after inserting the character so that the
/// overlay logic (in `actions`) can decide whether to open a popup.
pub fn is_slash_trigger(ch: char, col: usize) -> bool {
    ch == '/' && col == 0
}

pub fn is_mention_trigger(ch: char) -> bool {
    ch == '@'
}
