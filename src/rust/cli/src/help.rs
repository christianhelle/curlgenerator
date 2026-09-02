//! The help screen, laid out the way Spectre.Console.Cli renders it.

use unicode_width::UnicodeWidthStr;

const INDENT: &str = "    ";
const COLUMN_PADDING: usize = 4;
const FALLBACK_WIDTH: usize = 100;

const EXAMPLES: [&str; 7] = [
    "curlgenerator ./openapi.json",
    "curlgenerator ./openapi.json --output ./",
    "curlgenerator ./openapi.json --bash",
    "curlgenerator https://petstore.swagger.io/v2/swagger.json",
    "curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io",
    "curlgenerator ./openapi.json --authorization-header Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    "curlgenerator ./openapi.json --azure-scope [Some Application ID URI]/.default",
];

/// A single option row of the help screen.
struct Option_ {
    short: Option<char>,
    long: &'static str,
    value: Option<&'static str>,
    default: Option<&'static str>,
    description: &'static str,
}

fn options() -> Vec<Option_> {
    vec![
        Option_ {
            short: Some('h'),
            long: "help",
            value: None,
            default: None,
            description: "Prints help information",
        },
        Option_ {
            short: Some('v'),
            long: "version",
            value: None,
            default: None,
            description: "Prints version information",
        },
        Option_ {
            short: Some('o'),
            long: "output",
            value: Some("OUTPUT"),
            default: Some("./"),
            description: "Output directory",
        },
        Option_ {
            short: None,
            long: "bash",
            value: None,
            default: None,
            description: "Generate Bash scripts",
        },
        Option_ {
            short: None,
            long: "no-logging",
            value: None,
            default: None,
            description: "Don't log errors or collect telemetry",
        },
        Option_ {
            short: None,
            long: "skip-validation",
            value: None,
            default: None,
            description: "Skip validation of OpenAPI Specification file",
        },
        Option_ {
            short: None,
            long: "authorization-header",
            value: Some("HEADER"),
            default: None,
            description: "Authorization header to use for all requests",
        },
        Option_ {
            short: None,
            long: "content-type",
            value: Some("CONTENT-TYPE"),
            default: Some("application/json"),
            description: "Default Content-Type header to use for all requests",
        },
        Option_ {
            short: None,
            long: "base-url",
            value: Some("BASE-URL"),
            default: None,
            description: "Default Base URL to use for all requests. Use this if the OpenAPI spec doesn't explicitly specify a server URL",
        },
        Option_ {
            short: None,
            long: "azure-scope",
            value: Some("SCOPE"),
            default: None,
            description: "Azure Entra ID Scope to use for retrieving Access Token for Authorization header",
        },
        Option_ {
            short: None,
            long: "azure-tenant-id",
            value: Some("TENANT-ID"),
            default: None,
            description: "Azure Entra ID Tenant ID to use for retrieving Access Token for Authorization header",
        },
    ]
}

/// Renders the help screen for a terminal of the given width.
pub fn render(terminal_width: usize) -> String {
    let width = terminal_width.max(40);
    let options = options();

    let mut help = String::from("USAGE:\n");
    help.push_str(&format!(
        "{INDENT}curlgenerator [URL or input file] [OPTIONS]\n\n"
    ));

    help.push_str("EXAMPLES:\n");
    for example in EXAMPLES {
        help.push_str(&format!("{INDENT}{example}\n"));
    }

    help.push_str("\nARGUMENTS:\n");
    help.push_str(&format!(
        "{INDENT}[URL or input file]    URL or file path to OpenAPI Specification file\n"
    ));

    help.push_str("\nOPTIONS:\n");

    let names: Vec<String> = options.iter().map(render_name).collect();
    let name_column = INDENT.width()
        + names
            .iter()
            .map(|name| name.width())
            .max()
            .unwrap_or_default()
        + COLUMN_PADDING;
    let default_column = name_column
        + options
            .iter()
            .filter_map(|option| option.default)
            .map(str::width)
            .chain(std::iter::once("DEFAULT".width()))
            .max()
            .unwrap_or_default()
        + COLUMN_PADDING;
    let description_width = width.saturating_sub(default_column).max(1);

    help.push_str(&format!("{}DEFAULT\n", " ".repeat(name_column)));

    for (option, name) in options.iter().zip(names.iter()) {
        let default = option.default.unwrap_or("");
        let mut prefix = format!(
            "{INDENT}{name}{}{default}{}",
            " ".repeat(name_column - INDENT.width() - name.width()),
            " ".repeat(default_column - name_column - default.width())
        );

        for (index, chunk) in wrap(option.description, description_width)
            .into_iter()
            .enumerate()
        {
            if index > 0 {
                prefix = " ".repeat(default_column);
            }

            help.push_str(&prefix);
            help.push_str(&chunk);
            help.push('\n');
        }
    }

    help
}

/// Returns the terminal width, falling back to a fixed width when it is unknown.
pub fn terminal_width() -> usize {
    console::Term::stdout()
        .size_checked()
        .map(|(_, columns)| columns as usize)
        .unwrap_or(FALLBACK_WIDTH)
}

fn render_name(option: &Option_) -> String {
    let short = match option.short {
        Some(short) => format!("-{short}, "),
        None => "    ".to_string(),
    };
    let value = match option.value {
        Some(value) => format!(" <{value}>"),
        None => String::new(),
    };

    format!("{short}--{}{value}", option.long)
}

/// Wraps text on word boundaries.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.width() + 1 + word.width() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_the_spectre_section_order() {
        let help = render(120);
        let usage = help.find("USAGE:").unwrap();
        let examples = help.find("EXAMPLES:").unwrap();
        let arguments = help.find("ARGUMENTS:").unwrap();
        let options = help.find("OPTIONS:").unwrap();

        assert!(usage < examples);
        assert!(examples < arguments);
        assert!(arguments < options);
        assert!(help.contains("    curlgenerator [URL or input file] [OPTIONS]"));
        assert!(
            help.contains(
                "    [URL or input file]    URL or file path to OpenAPI Specification file"
            )
        );
    }

    #[test]
    fn aligns_the_default_column_across_rows() {
        let help = render(120);
        let default_header = help
            .lines()
            .find(|line| line.trim() == "DEFAULT")
            .expect("a DEFAULT header");
        let column = default_header.len() - "DEFAULT".len();

        let output_row = help
            .lines()
            .find(|line| line.contains("-o, --output <OUTPUT>"))
            .unwrap();
        let content_type_row = help
            .lines()
            .find(|line| line.contains("--content-type <CONTENT-TYPE>"))
            .unwrap();

        assert!(output_row[column..].starts_with("./"));
        assert!(content_type_row[column..].starts_with("application/json"));
    }

    #[test]
    fn lists_every_option() {
        let help = render(200);

        for long in [
            "--help",
            "--version",
            "--output",
            "--bash",
            "--no-logging",
            "--skip-validation",
            "--authorization-header",
            "--content-type",
            "--base-url",
            "--azure-scope",
            "--azure-tenant-id",
        ] {
            assert!(help.contains(long), "help should document {long}");
        }
    }

    #[test]
    fn wraps_long_descriptions_within_the_terminal_width() {
        let help = render(80);
        let options = help
            .split(
                "OPTIONS:
",
            )
            .nth(1)
            .expect("an options section");

        assert!(options.lines().all(|line| line.width() <= 80), "{options}");
        assert!(help.contains("Default Base URL"));
    }

    #[test]
    fn wraps_text_on_word_boundaries() {
        assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
        assert_eq!(wrap("", 10), vec![String::new()]);
    }
}
