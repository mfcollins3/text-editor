pub mod mention;
pub mod slash_command;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use mention::MentionProvider;
use slash_command::SlashCommand;

/// A popup overlay that can be rendered on top of the editor.
pub trait Overlay: Send + Sync {
    /// Render the overlay within `area`.
    fn render(&self, area: Rect, buf: &mut Buffer);
    /// Return `true` if this overlay should consume input events.
    fn is_active(&self) -> bool;
}

/// Manages the optional active overlay and the registered slash commands /
/// mention provider.
pub struct OverlayManager {
    active: Option<Box<dyn Overlay>>,
    slash_commands: Vec<SlashCommand>,
    mention_provider: Option<Box<dyn MentionProvider>>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            active: None,
            slash_commands: slash_command::default_commands(),
            mention_provider: None,
        }
    }

    /// Return `true` if an overlay is currently active.
    pub fn has_active(&self) -> bool {
        self.active
            .as_ref()
            .map(|o| o.is_active())
            .unwrap_or(false)
    }

    /// Close the active overlay (if any).
    pub fn close(&mut self) {
        self.active = None;
    }

    /// Register an additional slash command.
    pub fn register_slash_command(&mut self, command: SlashCommand) {
        self.slash_commands.push(command);
    }

    /// Set the mention provider.
    pub fn set_mention_provider(&mut self, provider: Box<dyn MentionProvider>) {
        self.mention_provider = Some(provider);
    }

    /// Render the active overlay (if any).
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if let Some(overlay) = &self.active {
            overlay.render(area, buf);
        }
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}
