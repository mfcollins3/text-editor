use ratatui::style::{Color, Modifier, Style};

/// A styled span within a single logical line.
#[derive(Debug, Clone)]
pub struct StyledRange {
    /// Start character column within the logical line.
    pub start_col: usize,
    /// End character column (exclusive).
    pub end_col: usize,
    pub style: Style,
}

/// Color theme for Markdown syntax highlighting.
#[derive(Debug, Clone)]
pub struct MarkdownTheme {
    pub heading: Style,
    pub bold: Style,
    pub italic: Style,
    pub code: Style,
    pub link: Style,
    pub blockquote: Style,
    pub list_marker: Style,
    pub hr: Style,
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self {
            heading: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            bold: Style::default().add_modifier(Modifier::BOLD),
            italic: Style::default().add_modifier(Modifier::ITALIC),
            code: Style::default().fg(Color::Green),
            link: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
            blockquote: Style::default().fg(Color::DarkGray),
            list_marker: Style::default().fg(Color::Cyan),
            hr: Style::default().fg(Color::DarkGray),
        }
    }
}

/// Compute syntax-highlighted spans for every line of `text`.
///
/// Returns a `Vec` with one entry per logical line.  Each entry is a list of
/// `StyledRange` values that should be applied on top of the default text
/// style.
pub fn highlight_buffer(text: &str, theme: &MarkdownTheme) -> Vec<Vec<StyledRange>> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut line_spans: Vec<Vec<StyledRange>> = text.lines().map(|_| Vec::new()).collect();
    // Ensure at least one entry exists for an empty document.
    if line_spans.is_empty() {
        line_spans.push(Vec::new());
    }

    let options = Options::all();
    let parser = Parser::new_ext(text, options);

    // Track the byte offset of each logical line start so we can map events
    // back to (line, col) pairs.
    let line_byte_offsets: Vec<usize> = {
        let mut offsets = vec![0usize];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                offsets.push(i + 1);
            }
        }
        offsets
    };

    let byte_to_line_col = |byte: usize| -> (usize, usize) {
        let line = line_byte_offsets
            .partition_point(|&o| o <= byte)
            .saturating_sub(1);
        let col = byte - line_byte_offsets[line];
        (line, col)
    };

    // Walk the event stream and produce styled ranges.
    // `pulldown_cmark` exposes ranges via `into_offset_iter`.
    let parser_with_offsets = Parser::new_ext(text, options).into_offset_iter();

    let mut active_style: Option<Style> = None;
    let mut heading_active = false;

    for (event, range) in parser_with_offsets {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                active_style = Some(theme.heading);
                heading_active = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                active_style = None;
                heading_active = false;
            }
            Event::Start(Tag::Strong) => {
                active_style = Some(theme.bold);
            }
            Event::End(TagEnd::Strong) => {
                active_style = None;
            }
            Event::Start(Tag::Emphasis) => {
                active_style = Some(theme.italic);
            }
            Event::End(TagEnd::Emphasis) => {
                active_style = None;
            }
            Event::Code(_) => {
                let style = theme.code;
                let (start_line, start_col) = byte_to_line_col(range.start);
                let (end_line, end_col) = byte_to_line_col(range.end);
                if start_line == end_line && start_line < line_spans.len() {
                    line_spans[start_line].push(StyledRange {
                        start_col,
                        end_col,
                        style,
                    });
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                active_style = Some(theme.blockquote);
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                active_style = None;
            }
            Event::Text(_) | Event::SoftBreak => {
                if let Some(style) = active_style {
                    let (start_line, start_col) = byte_to_line_col(range.start);
                    let (end_line, end_col) = byte_to_line_col(range.end);
                    if start_line < line_spans.len() {
                        // For simplicity span only the first line of multi-line
                        // runs; full multi-line support is a Phase 4 refinement.
                        let end = if start_line == end_line {
                            end_col
                        } else {
                            text.lines()
                                .nth(start_line)
                                .map(|l| l.len())
                                .unwrap_or(end_col)
                        };
                        line_spans[start_line].push(StyledRange {
                            start_col,
                            end_col: end,
                            style,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    line_spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_produces_styled_range() {
        let theme = MarkdownTheme::default();
        let spans = highlight_buffer("# Hello\n", &theme);
        // The heading line should have at least one styled range.
        assert!(
            !spans[0].is_empty(),
            "expected styled ranges for heading line"
        );
    }
}
