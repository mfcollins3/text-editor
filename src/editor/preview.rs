use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Render a Markdown document to a list of styled `ratatui` `Line`s suitable
/// for display in the preview pane.
pub fn render_preview(markdown: &str) -> Vec<Line<'static>> {
    use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();
    let mut list_depth: usize = 0;
    let mut ordered_list_counter: Vec<u64> = Vec::new();

    let push_line = |lines: &mut Vec<Line<'static>>, spans: &mut Vec<Span<'static>>| {
        lines.push(Line::from(std::mem::take(spans)));
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let style = match level {
                    HeadingLevel::H1 => Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                    HeadingLevel::H2 => Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                    _ => Style::default().add_modifier(Modifier::BOLD),
                };
                current_style = style;
            }
            Event::End(TagEnd::Heading(_)) => {
                push_line(&mut lines, &mut current_spans);
                lines.push(Line::default()); // blank line after heading
                current_style = Style::default();
            }
            Event::Start(Tag::Paragraph) => {
                current_style = Style::default();
            }
            Event::End(TagEnd::Paragraph) => {
                push_line(&mut lines, &mut current_spans);
                lines.push(Line::default());
            }
            Event::Start(Tag::Strong) => {
                current_style = current_style.add_modifier(Modifier::BOLD);
            }
            Event::End(TagEnd::Strong) => {
                current_style = current_style.remove_modifier(Modifier::BOLD);
            }
            Event::Start(Tag::Emphasis) => {
                current_style = current_style.add_modifier(Modifier::ITALIC);
            }
            Event::End(TagEnd::Emphasis) => {
                current_style = current_style.remove_modifier(Modifier::ITALIC);
            }
            Event::Start(Tag::List(start)) => {
                list_depth += 1;
                if let Some(n) = start {
                    ordered_list_counter.push(n);
                } else {
                    ordered_list_counter.push(0); // sentinel
                }
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
                ordered_list_counter.pop();
            }
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                let marker = if ordered_list_counter.last().copied().unwrap_or(0) == 0 {
                    format!("{indent}• ")
                } else {
                    let n = ordered_list_counter.last_mut().unwrap();
                    let label = format!("{indent}{}. ", n);
                    *n += 1;
                    label
                };
                current_spans.push(Span::styled(
                    marker,
                    Style::default().fg(Color::Cyan),
                ));
            }
            Event::End(TagEnd::Item) => {
                push_line(&mut lines, &mut current_spans);
            }
            Event::Code(text) => {
                current_spans.push(Span::styled(
                    text.to_string(),
                    Style::default().fg(Color::Green),
                ));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                current_style = Style::default().fg(Color::Green);
            }
            Event::End(TagEnd::CodeBlock) => {
                push_line(&mut lines, &mut current_spans);
                lines.push(Line::default());
                current_style = Style::default();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                current_style = Style::default().fg(Color::DarkGray);
                current_spans.push(Span::styled(
                    "┃ ".to_string(),
                    Style::default().fg(Color::DarkGray),
                ));
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                push_line(&mut lines, &mut current_spans);
                lines.push(Line::default());
                current_style = Style::default();
            }
            Event::Rule => {
                lines.push(Line::styled(
                    "─".repeat(40),
                    Style::default().fg(Color::DarkGray),
                ));
                lines.push(Line::default());
            }
            Event::SoftBreak => {
                current_spans.push(Span::raw(" "));
            }
            Event::HardBreak => {
                push_line(&mut lines, &mut current_spans);
            }
            Event::Text(text) => {
                current_spans.push(Span::styled(text.to_string(), current_style));
            }
            _ => {}
        }
    }

    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    lines
}
