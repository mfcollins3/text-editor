use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use super::Overlay;

/// The high-level category of action a slash command performs.
#[derive(Debug, Clone)]
pub enum SlashAction {
    /// Insert a Markdown block template at the current cursor position.
    InsertTemplate(String),
    /// Change the block style of the current paragraph.
    SetBlockStyle(BlockStyle),
    /// Asynchronous action (resolved by the host application).
    Async(String),
    /// Fully custom callback (host-supplied).
    Custom,
}

/// Markdown block styles that can be applied via a slash command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockStyle {
    Heading(u8),
    BulletList,
    NumberedList,
    Checkbox,
    Quote,
    CodeBlock,
    HorizontalRule,
}

/// A single entry in the slash command menu.
#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub action: SlashAction,
}

impl SlashCommand {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        icon: impl Into<String>,
        action: SlashAction,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            icon: icon.into(),
            action,
        }
    }
}

/// Return the built-in default slash commands.
pub fn default_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("h1", "Heading 1", "H1", SlashAction::SetBlockStyle(BlockStyle::Heading(1))),
        SlashCommand::new("h2", "Heading 2", "H2", SlashAction::SetBlockStyle(BlockStyle::Heading(2))),
        SlashCommand::new("h3", "Heading 3", "H3", SlashAction::SetBlockStyle(BlockStyle::Heading(3))),
        SlashCommand::new("bullet", "Bullet list", "•", SlashAction::SetBlockStyle(BlockStyle::BulletList)),
        SlashCommand::new("numbered", "Numbered list", "1.", SlashAction::SetBlockStyle(BlockStyle::NumberedList)),
        SlashCommand::new("todo", "Checkbox (todo)", "☐", SlashAction::SetBlockStyle(BlockStyle::Checkbox)),
        SlashCommand::new("code", "Fenced code block", "<>", SlashAction::SetBlockStyle(BlockStyle::CodeBlock)),
        SlashCommand::new("quote", "Block quote", "❝", SlashAction::SetBlockStyle(BlockStyle::Quote)),
        SlashCommand::new("divider", "Horizontal rule", "—", SlashAction::SetBlockStyle(BlockStyle::HorizontalRule)),
        SlashCommand::new("table", "Markdown table", "▦", SlashAction::InsertTemplate("| Col1 | Col2 |\n|------|------|\n| | |\n".into())),
    ]
}

/// Popup overlay for the slash-command picker.
pub struct SlashCommandOverlay {
    commands: Vec<SlashCommand>,
    filter: String,
    selected: usize,
}

impl SlashCommandOverlay {
    pub fn new(commands: Vec<SlashCommand>) -> Self {
        Self {
            commands,
            filter: String::new(),
            selected: 0,
        }
    }

    /// Update the filter string (typed characters after `/`).
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into().to_lowercase();
        self.selected = 0;
    }

    /// Return commands that match the current filter.
    pub fn filtered(&self) -> Vec<&SlashCommand> {
        if self.filter.is_empty() {
            self.commands.iter().collect()
        } else {
            self.commands
                .iter()
                .filter(|c| c.name.to_lowercase().starts_with(&self.filter))
                .collect()
        }
    }

    /// Move the selection down.
    pub fn select_next(&mut self) {
        let len = self.filtered().len();
        if len > 0 {
            self.selected = (self.selected + 1) % len;
        }
    }

    /// Move the selection up.
    pub fn select_prev(&mut self) {
        let len = self.filtered().len();
        if len > 0 {
            self.selected = (self.selected + len - 1) % len;
        }
    }

    /// Return the currently selected command (if any).
    pub fn selected_command(&self) -> Option<&SlashCommand> {
        self.filtered().get(self.selected).copied()
    }
}

impl Overlay for SlashCommandOverlay {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let items = self.filtered();
        let height = (items.len() as u16 + 2).min(area.height);
        let width = 36u16.min(area.width);
        let x = area.x;
        let y = area.y;

        let popup = Rect { x, y, width, height };

        Clear.render(popup, buf);
        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                Line::styled(format!(" {} {:12} {}", cmd.icon, cmd.name, cmd.description), style)
            })
            .collect();

        Paragraph::new(lines)
            .block(Block::bordered().title("Commands"))
            .render(popup, buf);
    }

    fn is_active(&self) -> bool {
        true
    }
}
