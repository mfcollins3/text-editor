use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::editor::{EditorResult, MarkdownEditor};
use crate::ui;

/// Top-level application state.
pub struct App {
    editor: MarkdownEditor,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            editor: MarkdownEditor::new(),
            should_quit: false,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        ui::draw(frame, &mut self.editor);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppSignal {
        match self.editor.handle_key_event(key) {
            EditorResult::Quit => {
                self.should_quit = true;
                AppSignal::Quit
            }
            EditorResult::Save => {
                let _ = self.editor.save();
                AppSignal::Continue
            }
            _ => AppSignal::Continue,
        }
    }
}

/// Signal returned after handling an input event.
pub enum AppSignal {
    Continue,
    Quit,
}

impl AppSignal {
    pub fn should_quit(&self) -> bool {
        matches!(self, AppSignal::Quit)
    }
}
