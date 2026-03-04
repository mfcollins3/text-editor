use std::path::PathBuf;

use crossterm::event::{Event, KeyEvent, MouseEvent};

pub mod actions;
pub mod buffer;
pub mod clipboard;
pub mod cursor;
pub mod highlight;
pub mod history;
pub mod input;
pub mod overlay;
pub mod preview;
pub mod recovery;
pub mod render;
pub mod viewport;
pub mod wrap;

use actions::EditorAction;
use buffer::Buffer;
use cursor::Cursor;
use history::History;
use input::map_key_event;
use overlay::mention::MentionProvider;
use overlay::slash_command::SlashCommand;
use overlay::OverlayManager;
use render::{EditorConfig, MarkdownEditorState};
use viewport::Viewport;

/// Result returned by the editor after processing an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorResult {
    /// The event was handled internally; no special host action is needed.
    Consumed,
    /// The user requested a save (`Ctrl+S`).
    Save,
    /// The user requested to quit (`Ctrl+Q`).
    Quit,
    /// The event was not recognised by the editor.
    Ignored,
}

/// Public facade for the Markdown editor component.
pub struct MarkdownEditor {
    state: MarkdownEditorState,
}

impl MarkdownEditor {
    /// Create a new, empty editor.
    pub fn new() -> Self {
        Self {
            state: MarkdownEditorState {
                buffer: Buffer::new(),
                cursor: Cursor::default(),
                history: History::new(),
                viewport: Viewport::default(),
                highlight_cache: Vec::new(),
                overlay: OverlayManager::new(),
                config: EditorConfig::default(),
            },
        }
    }

    /// Open a file for editing.
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let buffer = Buffer::from_file(path)?;
        Ok(Self {
            state: MarkdownEditorState {
                buffer,
                cursor: Cursor::default(),
                history: History::new(),
                viewport: Viewport::default(),
                highlight_cache: Vec::new(),
                overlay: OverlayManager::new(),
                config: EditorConfig::default(),
            },
        })
    }

    /// Initialise the buffer from a template string.
    pub fn with_template(template: impl Into<String>) -> Self {
        Self {
            state: MarkdownEditorState {
                buffer: Buffer::from_template(template),
                cursor: Cursor::default(),
                history: History::new(),
                viewport: Viewport::default(),
                highlight_cache: Vec::new(),
                overlay: OverlayManager::new(),
                config: EditorConfig::default(),
            },
        }
    }

    /// Return the current document text.
    pub fn get_text(&self) -> String {
        self.state.buffer.get_text()
    }

    /// Replace the entire document with new text.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.state.buffer.set_text(text);
        self.state.cursor = Cursor::default();
        self.state.history = History::new();
        self.state.highlight_cache.clear();
    }

    /// Process a raw crossterm event.
    pub fn handle_event(&mut self, event: Event) -> EditorResult {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            _ => EditorResult::Ignored,
        }
    }

    /// Process a key event.
    pub fn handle_key_event(&mut self, key: KeyEvent) -> EditorResult {
        if let Some(action) = map_key_event(key) {
            self.dispatch(action)
        } else {
            EditorResult::Ignored
        }
    }

    /// Process a mouse event.
    pub fn handle_mouse_event(&mut self, _mouse: MouseEvent) -> EditorResult {
        // Mouse handling will be implemented in Phase 2.
        EditorResult::Ignored
    }

    /// Return `true` if the buffer has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.state.buffer.is_dirty()
    }

    /// Save the buffer to its associated file path.
    pub fn save(&mut self) -> std::io::Result<()> {
        self.state.buffer.save()
    }

    /// Register a custom slash command.
    pub fn register_slash_command(&mut self, command: SlashCommand) {
        self.state.overlay.register_slash_command(command);
    }

    /// Supply a mention provider so that the `@` overlay can resolve names.
    pub fn set_mention_provider(&mut self, provider: Box<dyn MentionProvider>) {
        self.state.overlay.set_mention_provider(provider);
    }

    /// Show or hide the Markdown preview pane.
    pub fn set_preview_visible(&mut self, visible: bool) {
        self.state.config.show_preview = visible;
    }

    /// Show or hide line numbers.
    pub fn set_line_numbers_visible(&mut self, visible: bool) {
        self.state.config.show_line_numbers = visible;
    }

    /// Drive periodic tasks (auto-save recovery, etc.).
    pub fn tick(&mut self) {
        self.state.buffer.tick();
    }

    /// Expose the internal state so the renderer can borrow it.
    pub fn state(&self) -> &MarkdownEditorState {
        &self.state
    }

    /// Expose the internal state mutably so the renderer can borrow it.
    pub fn state_mut(&mut self) -> &mut MarkdownEditorState {
        &mut self.state
    }

    // --- private helpers ---

    fn dispatch(&mut self, action: EditorAction) -> EditorResult {
        actions::dispatch(action, &mut self.state)
    }
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        Self::new()
    }
}
