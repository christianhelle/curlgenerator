//! Spectre.Console-style terminal rendering: bordered panels and tables.
//!
//! Reproduces the rich console output of the original .NET tool, which used
//! the Spectre.Console NuGet package. Panels and tables use square box-drawing
//! borders; panel titles are embedded in the top border; colors are applied
//! with `owo-colors`. Column and panel widths are computed with `unicode-width`
//! so that borders line up around the emoji and text content.

use std::sync::OnceLock;

use owo_colors::{OwoColorize, Style};
use regex::Regex;
use unicode_width::UnicodeWidthStr;

/// Horizontal alignment of a table column.
#[derive(Debug, Clone, Copy)]
pub enum Align {
    Left,
    Right,
}

fn ansi() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new("\x1b\\[[0-9;]*m").unwrap())
}

/// Display width of a string, ignoring any embedded ANSI color escapes.
fn vis_width(text: &str) -> usize {
    UnicodeWidthStr::width(ansi().replace_all(text, "").as_ref())
}

fn paint(text: &str, style: Style) -> String {
    format!("{}", text.style(style))
}

/// Best-effort terminal width, defaulting to 80 columns when unavailable.
fn terminal_width() -> usize {
    terminal_size::terminal_size()
        .map(|(w, _)| w.0 as usize)
        .filter(|w| *w > 0)
        .unwrap_or(80)
}

/// A bordered table with a bold header row, mirroring Spectre's `Table`.
pub struct Table {
    border: Style,
    columns: Vec<Align>,
    headers: Vec<String>,
    rows: Vec<Vec<(String, Style)>>,
}

impl Table {
    pub fn new(border: Style) -> Self {
        Self {
            border,
            columns: Vec::new(),
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn column(mut self, header: &str, align: Align) -> Self {
        self.headers.push(header.to_string());
        self.columns.push(align);
        self
    }

    pub fn row(mut self, cells: Vec<(String, Style)>) -> Self {
        self.rows.push(cells);
        self
    }

    /// Renders the table to a vector of lines (with ANSI colors applied).
    pub fn render(&self) -> Vec<String> {
        let ncol = self.columns.len();
        let mut widths = vec![0usize; ncol];
        for (i, header) in self.headers.iter().enumerate() {
            widths[i] = vis_width(header);
        }
        for row in &self.rows {
            for (i, (text, _)) in row.iter().enumerate() {
                widths[i] = widths[i].max(vis_width(text));
            }
        }

        let hline = |left: &str, mid: &str, right: &str| -> String {
            let mut out = paint(left, self.border);
            for (i, w) in widths.iter().enumerate() {
                if i > 0 {
                    out.push_str(&paint(mid, self.border));
                }
                out.push_str(&paint(&"─".repeat(w + 2), self.border));
            }
            out.push_str(&paint(right, self.border));
            out
        };

        let bar = paint("│", self.border);
        let bold = Style::new().bold();

        let cell = |text: &str, style: Style, align: Align, w: usize| -> String {
            let pad = w.saturating_sub(vis_width(text));
            match align {
                Align::Left => format!(" {}{} ", paint(text, style), " ".repeat(pad)),
                Align::Right => format!(" {}{} ", " ".repeat(pad), paint(text, style)),
            }
        };

        let mut lines = Vec::new();
        lines.push(hline("┌", "┬", "┐"));

        let mut header_line = bar.clone();
        for (i, header) in self.headers.iter().enumerate() {
            header_line.push_str(&cell(header, bold, self.columns[i], widths[i]));
            header_line.push_str(&bar);
        }
        lines.push(header_line);
        lines.push(hline("├", "┼", "┤"));

        for row in &self.rows {
            let mut line = bar.clone();
            for (i, (text, style)) in row.iter().enumerate() {
                line.push_str(&cell(text, *style, self.columns[i], widths[i]));
                line.push_str(&bar);
            }
            lines.push(line);
        }

        lines.push(hline("└", "┴", "┘"));
        lines
    }
}

/// A bordered panel, optionally titled and/or expanded to the terminal width,
/// mirroring Spectre's `Panel`. Content lines may themselves contain ANSI
/// colors (for example, a rendered [`Table`]).
pub struct Panel {
    border: Style,
    title: Option<(String, Style)>,
    expand: bool,
    content: Vec<String>,
}

impl Panel {
    pub fn new(border: Style) -> Self {
        Self {
            border,
            title: None,
            expand: false,
            content: Vec::new(),
        }
    }

    pub fn title(mut self, text: &str, style: Style) -> Self {
        self.title = Some((text.to_string(), style));
        self
    }

    pub fn expand(mut self) -> Self {
        self.expand = true;
        self
    }

    pub fn content(mut self, lines: Vec<String>) -> Self {
        self.content = lines;
        self
    }

    pub fn line(mut self, line: String) -> Self {
        self.content.push(line);
        self
    }

    fn render(&self) -> Vec<String> {
        const PAD: usize = 1;
        let content_w = self.content.iter().map(|l| vis_width(l)).max().unwrap_or(0);
        let mut inner = content_w;

        let title_w = self.title.as_ref().map(|(t, _)| vis_width(t)).unwrap_or(0);
        if self.title.is_some() {
            // Top border layout: ┌ ─ <title> <dashes (>=1)> ┐
            let min_span = 1 + title_w + 1;
            if inner + 2 * PAD < min_span {
                inner = min_span - 2 * PAD;
            }
        }
        if self.expand {
            let target = terminal_width().saturating_sub(2);
            if target > inner + 2 * PAD {
                inner = target - 2 * PAD;
            }
        }

        let span = inner + 2 * PAD;
        let mut lines = Vec::new();

        let top = match &self.title {
            Some((text, style)) => {
                let dashes = span.saturating_sub(1 + title_w);
                format!(
                    "{}{}{}{}{}",
                    paint("┌", self.border),
                    paint("─", self.border),
                    paint(text, *style),
                    paint(&"─".repeat(dashes), self.border),
                    paint("┐", self.border),
                )
            }
            None => format!(
                "{}{}{}",
                paint("┌", self.border),
                paint(&"─".repeat(span), self.border),
                paint("┐", self.border),
            ),
        };
        lines.push(top);

        let bar = paint("│", self.border);
        for content in &self.content {
            let fill = inner.saturating_sub(vis_width(content));
            lines.push(format!(
                "{}{}{}{}{}{}",
                bar,
                " ".repeat(PAD),
                content,
                " ".repeat(fill),
                " ".repeat(PAD),
                bar,
            ));
        }

        lines.push(format!(
            "{}{}{}",
            paint("└", self.border),
            paint(&"─".repeat(span), self.border),
            paint("┘", self.border),
        ));
        lines
    }

    /// Prints the panel to standard output.
    pub fn print(&self) {
        for line in self.render() {
            println!("{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_borders_header_and_rows() {
        let table = Table::new(Style::new())
            .column("Component", Align::Left)
            .column("Count", Align::Right)
            .row(vec![
                ("Path Items".to_string(), Style::new()),
                ("13".to_string(), Style::new()),
            ]);
        let lines = table.render();
        // top, header, separator, one row, bottom
        assert_eq!(lines.len(), 5);
        assert!(lines[0].starts_with('┌') && lines[0].ends_with('┐'));
        assert!(lines[2].starts_with('├') && lines[2].ends_with('┤'));
        assert!(lines[4].starts_with('└') && lines[4].ends_with('┘'));
        assert!(lines[1].contains("Component"));
        assert!(lines[3].contains("Path Items"));
    }

    #[test]
    fn panel_embeds_title_in_top_border() {
        let panel = Panel::new(Style::new())
            .title("Configuration", Style::new())
            .content(vec!["hello".to_string()]);
        let lines = panel.render();
        assert!(lines[0].starts_with("┌─Configuration"));
        assert!(lines[0].ends_with('┐'));
        assert!(lines.last().unwrap().starts_with('└'));
        assert!(lines[1].contains("hello"));
    }

    #[test]
    fn untitled_panel_wraps_content() {
        let panel = Panel::new(Style::new()).line("abc".to_string());
        let lines = panel.render();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with('┌'));
        assert!(lines[1].contains("abc"));
    }

    #[test]
    fn vis_width_ignores_ansi() {
        let colored = format!("{}", "hello".style(Style::new().green()));
        assert_eq!(vis_width(&colored), 5);
    }
}
