//! Word wrapping of the embedded license documents into fixed-height visual
//! lines for the virtualized viewer. Lines are slices of the `'static` source,
//! so wrapping allocates only the line list (rebuilt when the width changes).

/// Columns a tab advances in the monospace font.
pub const TAB_COLUMNS: usize = 4;
/// Narrowest wrap width; narrower viewers scroll horizontally instead.
pub const MIN_COLUMNS: usize = 24;

/// How a visual line is styled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Body,
    /// Markdown heading (`#`, `##`, ...).
    Heading,
    /// Markdown code fence (```` ``` ````).
    Fence,
}

/// One row of the viewer: a slice of a source line, drawn `indent` columns in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VisualLine {
    pub text: &'static str,
    pub indent: u16,
    pub kind: LineKind,
}

fn columns_of(c: char) -> usize {
    if c == '\t' { TAB_COLUMNS } else { 1 }
}

fn kind_of(line: &str) -> LineKind {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        LineKind::Fence
    } else if trimmed.starts_with('#') {
        LineKind::Heading
    } else {
        LineKind::Body
    }
}

/// Continuation indent of a wrapped line: its leading whitespace, plus the
/// marker of a list item (`- `, `* `), at most half the width.
fn hanging_indent(line: &str, columns: usize) -> usize {
    let lead: usize = line
        .chars()
        .take_while(|c| c.is_whitespace())
        .map(columns_of)
        .sum();
    let rest = line.trim_start();
    let marker = if rest.starts_with("- ") || rest.starts_with("* ") {
        2
    } else {
        0
    };
    (lead + marker).min(columns / 2)
}

/// Wraps `text` at `columns` (at least [`MIN_COLUMNS`]): long lines break after
/// the last space that fits, or mid-word when a word is wider than the line;
/// continuation lines keep the line's hanging indent.
pub fn wrap(text: &'static str, columns: usize) -> Vec<VisualLine> {
    let columns = columns.max(MIN_COLUMNS);
    let mut out = Vec::with_capacity(text.len() / 48);
    for line in text.lines() {
        let kind = kind_of(line);
        let indent = hanging_indent(line, columns);
        let mut rest = line;
        let mut first = true;
        loop {
            let pad = if first { 0 } else { indent };
            let (head, tail) = split_at_width(rest, columns - pad);
            out.push(VisualLine {
                text: head,
                indent: u16::try_from(pad).unwrap_or(0),
                kind,
            });
            if tail.is_empty() {
                break;
            }
            rest = tail;
            first = false;
        }
    }
    out
}

/// Splits off the longest head of `line` that fits `width` columns, breaking
/// after a space when possible; the tail starts at the next non-space.
fn split_at_width(line: &'static str, width: usize) -> (&'static str, &'static str) {
    let mut used = 0;
    let mut last_space = None;
    for (i, c) in line.char_indices() {
        used += columns_of(c);
        if used > width {
            // Never cut at 0, so every call makes progress.
            let cut = match last_space {
                Some(space) if space > 0 => space,
                _ if i > 0 => i,
                _ => c.len_utf8(),
            };
            let (head, tail) = line.split_at(cut);
            return (head.trim_end(), tail.trim_start());
        }
        if c == ' ' {
            last_space = Some(i);
        }
    }
    (line, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(lines: &[VisualLine]) -> Vec<&str> {
        lines.iter().map(|l| l.text).collect()
    }

    #[test]
    fn bundled_documents_wrap_within_the_width_without_losing_text() {
        let non_space = |t: &str| t.chars().filter(|c| !c.is_whitespace()).collect::<String>();
        for text in [
            super::super::licenses::THIRD_PARTY_LICENSES,
            crate::utils::colormap::LICENSES_TEXT,
        ] {
            for columns in [MIN_COLUMNS, 60, 100] {
                let lines = wrap(text, columns);
                for line in &lines {
                    let used: usize = line.text.chars().map(columns_of).sum();
                    assert!(usize::from(line.indent) + used <= columns, "{line:?}");
                }
                let joined: String = lines.iter().map(|l| l.text).collect();
                assert_eq!(non_space(&joined), non_space(text), "{columns} columns");
            }
        }
    }

    #[test]
    fn short_lines_pass_through_with_their_kind() {
        let lines = wrap("# Title\n\nplain\n```text\n", 40);
        assert_eq!(texts(&lines), ["# Title", "", "plain", "```text"]);
        let kinds: Vec<_> = lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            [
                LineKind::Heading,
                LineKind::Body,
                LineKind::Body,
                LineKind::Fence
            ]
        );
    }

    #[test]
    fn long_lines_break_at_spaces_within_the_width() {
        let text = "the quick brown fox jumps over the lazy dog again and again";
        let lines = wrap(text, 24);
        assert_eq!(
            texts(&lines),
            [
                "the quick brown fox",
                "jumps over the lazy dog",
                "again and again"
            ]
        );
        assert!(lines.iter().all(|l| l.text.chars().count() <= 24));
        assert_eq!(texts(&lines).join(" "), text, "no text is lost");
    }

    #[test]
    fn words_wider_than_the_line_break_mid_word() {
        let word: &'static str = "abcdefghijklmnopqrstuvwxyzabcdefghij";
        let lines = wrap(word, 24);
        assert_eq!(texts(&lines), ["abcdefghijklmnopqrstuvwx", "yzabcdefghij"]);
    }

    #[test]
    fn list_items_wrap_with_a_hanging_indent() {
        let lines = wrap("  - one two three four five six seven eight", 24);
        assert_eq!(lines[0].indent, 0);
        assert_eq!(lines[0].text, "  - one two three four");
        assert!(lines[1..].iter().all(|l| l.indent == 4), "{lines:?}");
    }

    #[test]
    fn tabs_count_as_four_columns_and_multibyte_text_splits_on_char_boundaries() {
        let lines = wrap("\tééééééééééééééééééééééééé", 24);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| !l.text.is_empty()));
    }
}
