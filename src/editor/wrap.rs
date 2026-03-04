use unicode_width::UnicodeWidthChar;

/// A single visual (wrapped) line derived from a logical buffer line.
#[derive(Debug, Clone)]
pub struct VisualLine {
    /// Index of the logical (unwrapped) buffer line.
    pub logical_line: usize,
    /// Character index of the first character of this visual line within the
    /// logical line.
    pub start_col: usize,
    /// Character index one past the last character of this visual line within
    /// the logical line.
    pub end_col: usize,
    /// `true` for the first visual line of a logical line.
    pub is_first: bool,
}

/// Compute the visual (word-wrapped) lines for a single logical line of text.
///
/// `max_width` is the number of display columns available.  The function
/// splits the line at word boundaries so that each visual line fits within
/// `max_width` columns.
pub fn wrap_line(logical_line: usize, text: &str, max_width: usize) -> Vec<VisualLine> {
    if max_width == 0 {
        return vec![VisualLine {
            logical_line,
            start_col: 0,
            end_col: text.chars().count(),
            is_first: true,
        }];
    }

    let chars: Vec<char> = text.trim_end_matches('\n').chars().collect();
    if chars.is_empty() {
        return vec![VisualLine {
            logical_line,
            start_col: 0,
            end_col: 0,
            is_first: true,
        }];
    }

    let mut lines = Vec::new();
    let mut line_start = 0;
    let mut current_width: usize = 0;
    let mut last_space: Option<usize> = None;
    let mut is_first = true;

    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        let w = ch.width().unwrap_or(1);

        if ch == ' ' || ch == '\t' {
            last_space = Some(i);
        }

        if current_width + w > max_width {
            // Try to break at the last space.
            let break_at = last_space.unwrap_or(i);
            let end_col = if break_at == line_start { i } else { break_at };

            lines.push(VisualLine {
                logical_line,
                start_col: line_start,
                end_col,
                is_first,
            });

            // Skip the space character itself when wrapping at a space.
            line_start = if break_at == i {
                end_col
            } else {
                end_col + 1
            };
            i = line_start;
            current_width = 0;
            last_space = None;
            is_first = false;
        } else {
            current_width += w;
            i += 1;
        }
    }

    // Remainder.
    lines.push(VisualLine {
        logical_line,
        start_col: line_start,
        end_col: chars.len(),
        is_first,
    });

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wrap_needed() {
        let lines = wrap_line(0, "hello", 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].start_col, 0);
        assert_eq!(lines[0].end_col, 5);
        assert!(lines[0].is_first);
    }

    #[test]
    fn wraps_at_word_boundary() {
        // "hello world" with max_width=7 should wrap after "hello"
        let lines = wrap_line(0, "hello world", 7);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].end_col, 5); // "hello"
        assert_eq!(lines[1].start_col, 6); // "world"
        assert!(lines[0].is_first);
        assert!(!lines[1].is_first);
    }
}
