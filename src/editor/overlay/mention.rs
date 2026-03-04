use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use super::Overlay;

/// The kind of entity that can be mentioned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentionKind {
    User,
    AiAgent,
    Service,
}

/// A single mentionable entity returned by a `MentionProvider`.
#[derive(Debug, Clone)]
pub struct Mentionable {
    pub id: String,
    pub display_name: String,
    pub kind: MentionKind,
    pub description: Option<String>,
}

/// Trait that the host application implements to supply mention candidates.
pub trait MentionProvider: Send + Sync {
    /// Return all entities whose `display_name` starts with `prefix`.
    fn query(&self, prefix: &str) -> Vec<Mentionable>;
}

/// Popup overlay for the `@`-mention picker.
pub struct MentionOverlay {
    items: Vec<Mentionable>,
    selected: usize,
    filter: String,
}

impl MentionOverlay {
    pub fn new(items: Vec<Mentionable>) -> Self {
        Self {
            items,
            selected: 0,
            filter: String::new(),
        }
    }

    /// Update the filter string (typed characters after `@`).
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into().to_lowercase();
        self.selected = 0;
    }

    /// Return items that match the current filter.
    pub fn filtered(&self) -> Vec<&Mentionable> {
        if self.filter.is_empty() {
            self.items.iter().collect()
        } else {
            self.items
                .iter()
                .filter(|m| m.display_name.to_lowercase().starts_with(&self.filter))
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

    /// Return the currently highlighted mention (if any).
    pub fn selected_item(&self) -> Option<&Mentionable> {
        self.filtered().get(self.selected).copied()
    }
}

impl Overlay for MentionOverlay {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let items = self.filtered();
        let height = (items.len() as u16 + 2).min(area.height);
        let width = 36u16.min(area.width);
        let popup = Rect {
            x: area.x,
            y: area.y,
            width,
            height,
        };

        Clear.render(popup, buf);
        let lines: Vec<Line> = items
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let kind_label = match m.kind {
                    MentionKind::User => "👤",
                    MentionKind::AiAgent => "🤖",
                    MentionKind::Service => "⚙",
                };
                let style = if i == self.selected {
                    Style::default().bg(Color::Blue).fg(Color::White)
                } else {
                    Style::default()
                };
                Line::styled(format!(" {kind_label} {}", m.display_name), style)
            })
            .collect();

        Paragraph::new(lines)
            .block(Block::bordered().title("Mention"))
            .render(popup, buf);
    }

    fn is_active(&self) -> bool {
        true
    }
}
