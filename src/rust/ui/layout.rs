//! Box drawing primitives that mirror Spectre.Console panels and tables.

use super::width::{char_width, emoji_presentation_width};

/// The width used when the terminal size is unknown, matching Spectre.Console.
pub const DEFAULT_WIDTH: usize = 80;

/// How a table cell is aligned inside its column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    /// Aligned against the left edge of the column.
    Left,
    /// Aligned against the right edge of the column.
    Right,
}

/// A single table cell.
#[derive(Debug, Clone)]
pub struct Cell {
    text: String,
    style: Vec<&'static str>,
}

impl Cell {
    /// Creates an unstyled cell.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Vec::new(),
        }
    }

    /// Creates a cell rendered with the given ANSI style codes.
    pub fn styled(text: impl Into<String>, style: &[&'static str]) -> Self {
        Self {
            text: text.into(),
            style: style.to_vec(),
        }
    }

    fn width(&self) -> usize {
        display_width(&self.text)
    }
}

/// A table with a header row and a fixed number of columns.
#[derive(Debug, Clone)]
pub struct Table {
    headers: Vec<Cell>,
    alignments: Vec<Alignment>,
    rows: Vec<Vec<Cell>>,
    border: Vec<&'static str>,
}

impl Table {
    /// Creates a table with the given headers, alignments, and border style.
    pub fn new(headers: Vec<Cell>, alignments: Vec<Alignment>, border: &[&'static str]) -> Self {
        Self {
            headers,
            alignments,
            rows: Vec::new(),
            border: border.to_vec(),
        }
    }

    /// Appends a row.
    pub fn push(&mut self, row: Vec<Cell>) {
        self.rows.push(row);
    }

    /// Returns `true` when the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Renders the table, shrinking the last column so the result fits `available` columns.
    pub fn render(&self, available: usize, colors: bool) -> Vec<String> {
        let widths = self.column_widths(available);
        let border = |text: &str| style(text, &self.border, colors);

        let mut lines = vec![border(&rule("┌", "┬", "┐", &widths))];
        lines.extend(self.render_row(&self.headers, &widths, colors, true));
        lines.push(border(&rule("├", "┼", "┤", &widths)));

        for row in &self.rows {
            lines.extend(self.render_row(row, &widths, colors, false));
        }

        lines.push(border(&rule("└", "┴", "┘", &widths)));
        lines
    }

    /// Returns the rendered width of the table for the given available width.
    pub fn width(&self, available: usize) -> usize {
        table_width(&self.column_widths(available))
    }

    fn column_widths(&self, available: usize) -> Vec<usize> {
        let mut widths: Vec<usize> = self
            .headers
            .iter()
            .enumerate()
            .map(|(column, header)| {
                self.rows
                    .iter()
                    .filter_map(|row| row.get(column))
                    .map(Cell::width)
                    .chain(std::iter::once(header.width()))
                    .max()
                    .unwrap_or_default()
            })
            .collect();

        let mut total = table_width(&widths);
        if total > available
            && let Some(last) = widths.last_mut()
        {
            let overflow = total - available;
            *last = last.saturating_sub(overflow).max(1);
            total = table_width(&widths);
            let _ = total;
        }

        widths
    }

    fn render_row(
        &self,
        row: &[Cell],
        widths: &[usize],
        colors: bool,
        header: bool,
    ) -> Vec<String> {
        let wrapped: Vec<Vec<String>> = widths
            .iter()
            .enumerate()
            .map(|(column, width)| match row.get(column) {
                Some(cell) => hard_wrap(&cell.text, *width),
                None => vec![String::new()],
            })
            .collect();
        let height = wrapped.iter().map(Vec::len).max().unwrap_or(1);
        let separator = style("│", &self.border, colors);

        (0..height)
            .map(|line| {
                let mut rendered = separator.clone();

                for (column, width) in widths.iter().enumerate() {
                    let text = wrapped[column].get(line).cloned().unwrap_or_default();
                    let padding = width.saturating_sub(display_width(&text));
                    let alignment = self
                        .alignments
                        .get(column)
                        .copied()
                        .unwrap_or(Alignment::Left);
                    let cell_style: Vec<&'static str> = if header {
                        vec!["1"]
                    } else {
                        row.get(column)
                            .map(|cell| cell.style.clone())
                            .unwrap_or_default()
                    };
                    let styled = style(&text, &cell_style, colors);

                    match alignment {
                        Alignment::Left => {
                            rendered.push_str(&format!(" {styled}{} ", " ".repeat(padding)))
                        }
                        Alignment::Right => {
                            rendered.push_str(&format!(" {}{styled} ", " ".repeat(padding)))
                        }
                    }

                    rendered.push_str(&separator);
                }

                rendered
            })
            .collect()
    }
}

/// Renders a bordered panel around the given content lines.
///
/// The optional header is drawn inside the top border, the way Spectre.Console renders it.
pub fn panel(
    header: Option<(&str, &[&'static str])>,
    content: &[String],
    inner_width: usize,
    border: &[&'static str],
    colors: bool,
) -> Vec<String> {
    let width = inner_width + 2;
    let top = match header {
        Some((text, header_style)) => format!(
            "{}{}{}",
            style("┌─", border, colors),
            style(text, header_style, colors),
            style(
                &format!(
                    "{}┐",
                    "─".repeat(width.saturating_sub(1 + display_width(text)))
                ),
                border,
                colors
            )
        ),
        None => style(&format!("┌{}┐", "─".repeat(width)), border, colors),
    };

    let mut lines = vec![top];
    let separator = style("│", border, colors);

    for line in content {
        let padding = inner_width.saturating_sub(visible_width(line));
        lines.push(format!(
            "{separator} {line}{} {separator}",
            " ".repeat(padding)
        ));
    }

    lines.push(style(&format!("└{}┘", "─".repeat(width)), border, colors));
    lines
}

/// Applies ANSI style codes when colors are enabled.
pub fn style(text: &str, codes: &[&str], colors: bool) -> String {
    if !colors || codes.is_empty() || text.is_empty() {
        return text.to_string();
    }

    format!("\u{1b}[{}m{text}\u{1b}[0m", codes.join(";"))
}

/// Returns the display width of a string, ignoring ANSI escape sequences.
pub fn visible_width(text: &str) -> usize {
    display_width(&strip_ansi(text))
}

/// Returns the display width of a string.
///
/// A character followed by the emoji variation selector (`U+FE0F`) is measured together with
/// it, since that is how terminals actually render it: as a single, double-width emoji glyph.
/// Measuring the pair separately (one column for the base character, zero for the selector)
/// undercounts it by one column and misaligns any row that contains it.
pub fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut characters = text.chars().peekable();

    while let Some(character) = characters.next() {
        if characters.peek() == Some(&'\u{fe0f}') {
            characters.next();
            width += emoji_presentation_width(character);
        } else {
            width += char_width(character);
        }
    }

    width
}

/// Removes ANSI escape sequences from a string.
pub fn strip_ansi(text: &str) -> String {
    let mut plain = String::with_capacity(text.len());
    let mut characters = text.chars();

    while let Some(character) = characters.next() {
        if character != '\u{1b}' {
            plain.push(character);
            continue;
        }

        for escape in characters.by_ref() {
            if escape == 'm' {
                break;
            }
        }
    }

    plain
}

/// Splits text into chunks of at most `width` display columns.
pub fn hard_wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for character in text.chars() {
        let character_width = char_width(character);
        if display_width(&current) + character_width > width {
            lines.push(std::mem::take(&mut current));
        }

        current.push(character);
    }

    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }

    lines
}

fn rule(start: &str, middle: &str, end: &str, widths: &[usize]) -> String {
    let segments: Vec<String> = widths.iter().map(|width| "─".repeat(width + 2)).collect();

    format!("{start}{}{end}", segments.join(middle))
}

fn table_width(widths: &[usize]) -> usize {
    widths.iter().map(|width| width + 2).sum::<usize>() + widths.len() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats_table() -> Table {
        let mut table = Table::new(
            vec![Cell::new("Component"), Cell::new("Count")],
            vec![Alignment::Left, Alignment::Right],
            &["32"],
        );
        table.push(vec![Cell::new("📝 Path Items"), Cell::new("2")]);
        table.push(vec![Cell::new("📦 Request Bodies"), Cell::new("15")]);
        table
    }

    #[test]
    fn renders_a_table_with_aligned_columns() {
        let lines = stats_table().render(DEFAULT_WIDTH, false);

        assert_eq!(
            lines,
            vec![
                "┌───────────────────┬───────┐",
                "│ Component         │ Count │",
                "├───────────────────┼───────┤",
                "│ 📝 Path Items     │     2 │",
                "│ 📦 Request Bodies │    15 │",
                "└───────────────────┴───────┘",
            ]
        );
    }

    #[test]
    fn reports_the_rendered_table_width() {
        assert_eq!(stats_table().width(DEFAULT_WIDTH), 29);
    }

    #[test]
    fn shrinks_and_wraps_the_last_column_to_fit() {
        let mut table = Table::new(
            vec![Cell::new("Setting"), Cell::new("Value")],
            vec![Alignment::Left, Alignment::Left],
            &["90"],
        );
        table.push(vec![Cell::new("Source"), Cell::new("abcdefghij")]);

        let lines = table.render(20, false);

        assert_eq!(
            lines,
            vec![
                "┌─────────┬────────┐",
                "│ Setting │ Value  │",
                "├─────────┼────────┤",
                "│ Source  │ abcdef │",
                "│         │ ghij   │",
                "└─────────┴────────┘",
            ]
        );
    }

    #[test]
    fn draws_a_panel_with_a_header_inside_the_top_border() {
        let lines = panel(
            Some(("📊 Stats", &["1", "34"][..])),
            &["hello".to_string()],
            12,
            &["34"],
            false,
        );

        assert_eq!(
            lines,
            vec!["┌─📊 Stats─────┐", "│ hello        │", "└──────────────┘",]
        );
    }

    #[test]
    fn draws_a_panel_without_a_header() {
        let lines = panel(None, &["hi".to_string()], 4, &["32"], false);

        assert_eq!(lines, vec!["┌──────┐", "│ hi   │", "└──────┘"]);
    }

    #[test]
    fn applies_ansi_codes_only_when_colors_are_enabled() {
        assert_eq!(style("text", &["32"], true), "\u{1b}[32mtext\u{1b}[0m");
        assert_eq!(style("text", &["32"], false), "text");
        assert_eq!(style("text", &[], true), "text");
    }

    #[test]
    fn measures_width_without_escape_sequences() {
        assert_eq!(visible_width("\u{1b}[32mtext\u{1b}[0m"), 4);
        assert_eq!(strip_ansi("\u{1b}[1;32mok\u{1b}[0m"), "ok");
    }

    #[test]
    fn measures_width_one_character_at_a_time() {
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("\u{1f4dd} Path Items"), 13);
        // A character followed by the emoji variation selector renders as one double-width
        // glyph, so the pair is measured together rather than as 1 (base) + 0 (selector).
        assert_eq!(display_width("\u{2699}\u{fe0f} Operations"), 13);
        assert_eq!(visible_width("\u{1b}[32m\u{2699}\u{fe0f}\u{1b}[0m"), 2);
    }

    #[test]
    fn hard_wraps_on_display_columns() {
        assert_eq!(hard_wrap("abcdef", 4), vec!["abcd", "ef"]);
        assert_eq!(hard_wrap("", 4), vec![String::new()]);
        assert_eq!(hard_wrap("abc", 0), vec![String::new()]);
    }

    #[test]
    fn measures_wide_zero_width_and_text_presentation_characters() {
        assert_eq!(display_width("日本語"), 6);
        assert_eq!(display_width("e\u{301}"), 1);
        assert_eq!(display_width("a\u{200d}b"), 2);
        assert_eq!(display_width("\t"), 1);
        assert_eq!(display_width("\u{26a0}"), 1);
        assert_eq!(display_width("\u{26a0}\u{fe0f}"), 2);
        assert_eq!(display_width("\u{23f1}\u{fe0f} Duration"), 11);
        assert_eq!(hard_wrap("日本語", 4), vec!["日本", "語"]);
    }
}
