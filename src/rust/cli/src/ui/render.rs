//! The individual screens the CLI prints while generating scripts.

use std::time::Duration;

use curlgenerator_core::{ScriptFile, openapi::OpenApiStats};

use crate::ui::layout::{Alignment, Cell, Table, panel, style, visible_width};

/// ANSI style codes used by the screens, matching the Spectre.Console colors of the legacy CLI.
pub mod color {
    /// Bold text.
    pub const BOLD: &str = "1";
    /// Dimmed text.
    pub const DIM: &str = "2";
    /// Red text.
    pub const RED: &str = "31";
    /// Green text.
    pub const GREEN: &str = "32";
    /// Yellow text.
    pub const YELLOW: &str = "33";
    /// Blue text.
    pub const BLUE: &str = "34";
    /// Cyan text.
    pub const CYAN: &str = "36";
    /// Grey text.
    pub const GREY: &str = "90";
}

/// The configuration values shown in the configuration panel.
#[derive(Debug, Clone)]
pub struct ConfigurationView<'a> {
    /// The OpenAPI path or URL.
    pub open_api_path: &'a str,
    /// The output directory.
    pub output: &'a str,
    /// The default content type.
    pub content_type: &'a str,
    /// The configured base URL.
    pub base_url: Option<&'a str>,
    /// Whether Bash scripts are generated.
    pub bash: bool,
    /// Whether validation is skipped.
    pub skip_validation: bool,
    /// The configured authorization header.
    pub authorization_header: Option<&'a str>,
}

/// Renders the application header panel and the support key line.
pub fn header(version: &str, no_logging: bool, width: usize, colors: bool) -> String {
    let title = format!("🔧 cURL Request Generator v{version}");
    let inner = width.saturating_sub(4).max(visible_width(&title));
    let content = vec![style(&title, &[color::GREEN, color::BOLD], colors)];
    let support = if no_logging {
        style(
            "⚠️  Unavailable when logging is disabled",
            &[color::YELLOW],
            colors,
        )
    } else {
        style(
            &format!("🔑 Support key: {}", curlgenerator_core::support_key()),
            &[color::GREEN],
            colors,
        )
    };

    let mut lines = panel(None, &content, inner, &[color::GREEN], colors);
    lines.push(String::new());
    lines.push(support);
    lines.push(String::new());

    join(&lines)
}

/// Renders the configuration panel.
pub fn configuration(view: &ConfigurationView<'_>, width: usize, colors: bool) -> String {
    let mut table = settings_table("Setting", "Value", Alignment::Left, color::GREY);

    table.push(row("📁 OpenAPI Source", view.open_api_path, color::CYAN));
    table.push(row("📂 Output Folder", view.output, color::CYAN));
    table.push(row("🌐 Content Type", view.content_type, color::CYAN));

    if let Some(base_url) = view.base_url.filter(|value| !value.trim().is_empty()) {
        table.push(row("🔗 Base URL", base_url, color::CYAN));
    }

    if view.bash {
        table.push(row("🐚 Bash Scripts", "✓ Enabled", color::GREEN));
    }

    if view.skip_validation {
        table.push(row("⚠️  Validation", "⚠️  Skipped", color::YELLOW));
    }

    if let Some(authorization) = view
        .authorization_header
        .filter(|value| !value.trim().is_empty())
    {
        table.push(row(
            "🔐 Authorization",
            &truncate(authorization),
            color::DIM,
        ));
    }

    wrap_in_panel(&table, "📋 Configuration", color::YELLOW, width, colors)
}

/// Renders the OpenAPI statistics panel.
pub fn statistics(stats: &OpenApiStats, width: usize, colors: bool) -> String {
    let mut table = settings_table("Component", "Count", Alignment::Right, color::BLUE);

    for (label, count) in [
        ("📝 Path Items", stats.path_item_count),
        ("⚙️  Operations", stats.operation_count),
        ("📝 Parameters", stats.parameter_count),
        ("📦 Request Bodies", stats.request_body_count),
        ("📋 Responses", stats.response_count),
        ("🔗 Links", stats.link_count),
        ("📞 Callbacks", stats.callback_count),
        ("📝 Schemas", stats.schema_count),
    ] {
        table.push(vec![
            Cell::new(label),
            Cell::styled(count.to_string(), &[color::BLUE]),
        ]);
    }

    wrap_in_panel(&table, "📊 OpenAPI Statistics", color::BLUE, width, colors)
}

/// Renders the results panel and the list of generated files.
pub fn results(
    files: &[ScriptFile],
    elapsed: Duration,
    output_location: &str,
    width: usize,
    colors: bool,
) -> String {
    let mut table = settings_table("Metric", "Value", Alignment::Left, color::GREEN);

    table.push(row(
        "📄 Files Generated",
        &files.len().to_string(),
        color::GREEN,
    ));
    table.push(row(
        "⏱️  Duration",
        &format!("{:.0}ms", elapsed.as_secs_f64() * 1000.0),
        color::GREEN,
    ));
    table.push(row("📁 Output Location", output_location, color::CYAN));

    let (title, border) = if files.is_empty() {
        ("⚠️  Generation Complete (No Files)", color::YELLOW)
    } else {
        ("✅ Generation Complete", color::GREEN)
    };

    let mut output = wrap_in_panel(&table, title, border, width, colors);

    if !files.is_empty() {
        output.push_str(&style(
            "📁 Generated Files:",
            &[color::BOLD, color::YELLOW],
            colors,
        ));
        output.push('\n');

        for file in files {
            output.push_str(&format!(
                "  📝 {} {}\n",
                style(&file.filename, &[color::CYAN], colors),
                style(
                    &format!("({})", file_size(file.content.len())),
                    &[color::DIM],
                    colors
                )
            ));
        }
    }

    output.push('\n');
    output.push_str(&style("🎉 Done!", &[color::BOLD, color::GREEN], colors));
    output.push('\n');
    output
}

/// Renders an error message.
pub fn error(message: &str, colors: bool) -> String {
    format!(
        "\n{}\n",
        style(&format!("Error:\n{message}"), &[color::RED], colors)
    )
}

/// Renders the hint shown when a specification version cannot be validated.
pub fn unsupported_version_tips(colors: bool) -> String {
    let tips = "Tips:\n\
        Consider using the --skip-validation argument.\n\
        In some cases, the features that are specific to the unsupported versions of \
        OpenAPI specifications aren't really used.\n";

    format!("\n{}\n", style(tips, &[color::YELLOW], colors))
}

/// Renders a single validation diagnostic.
pub fn diagnostic(label: &str, message: &str, code: &'static str, colors: bool) -> String {
    format!(
        "{}\n",
        style(&format!("{label}:\n{message}\n"), &[code], colors)
    )
}

/// Renders the message shown while an Azure Entra ID token is acquired.
pub fn azure_started(colors: bool) -> String {
    format!(
        "{}\n",
        style(
            "🔐 Acquiring authorization header from Azure Entra ID...",
            &[color::GREEN],
            colors
        )
    )
}

/// Renders the message shown once an Azure Entra ID token was acquired.
pub fn azure_succeeded(colors: bool) -> String {
    format!(
        "{}\n",
        style(
            "✅ Successfully acquired access token",
            &[color::GREEN],
            colors
        )
    )
}

fn settings_table(left: &str, right: &str, alignment: Alignment, border: &'static str) -> Table {
    Table::new(
        vec![Cell::new(left), Cell::new(right)],
        vec![Alignment::Left, alignment],
        &[border],
    )
}

fn row(label: &str, value: &str, code: &'static str) -> Vec<Cell> {
    vec![Cell::new(label), Cell::styled(value, &[code])]
}

fn wrap_in_panel(
    table: &Table,
    title: &str,
    border: &'static str,
    width: usize,
    colors: bool,
) -> String {
    let available = width.saturating_sub(4);
    let inner = table.width(available).max(visible_width(title) + 2);
    let lines = panel(
        Some((title, &[color::BOLD, border])),
        &table.render(available, colors),
        inner,
        &[border],
        colors,
    );

    format!("{}\n\n", join(&lines))
}

fn truncate(value: &str) -> String {
    if value.chars().count() > 50 {
        format!("{}...", value.chars().take(47).collect::<String>())
    } else {
        value.to_string()
    }
}

fn file_size(bytes: usize) -> String {
    match bytes {
        bytes if bytes < 1024 => format!("{bytes} bytes"),
        bytes if bytes < 1024 * 1024 => format!("{:.1} KB", bytes as f64 / 1024.0),
        bytes => format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)),
    }
}

fn join(lines: &[String]) -> String {
    let mut joined = lines.join("\n");
    joined.push('\n');
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view<'a>() -> ConfigurationView<'a> {
        ConfigurationView {
            open_api_path: "./openapi.json",
            output: "./",
            content_type: "application/json",
            base_url: None,
            bash: false,
            skip_validation: false,
            authorization_header: None,
        }
    }

    #[test]
    fn renders_the_header_panel_and_support_key() {
        let output = header("0.4.1", false, 80, false);

        assert!(output.starts_with("┌"));
        assert!(output.contains("│ 🔧 cURL Request Generator v0.4.1"));
        assert!(output.contains("🔑 Support key: "));
    }

    #[test]
    fn hides_the_support_key_when_logging_is_disabled() {
        let output = header("0.4.1", true, 80, false);

        assert!(output.contains("⚠️  Unavailable when logging is disabled"));
        assert!(!output.contains("Support key"));
    }

    #[test]
    fn renders_only_the_configured_settings() {
        let output = configuration(&view(), 80, false);

        assert!(output.contains("┌─📋 Configuration"));
        assert!(output.contains("📁 OpenAPI Source"));
        assert!(output.contains("./openapi.json"));
        assert!(!output.contains("Base URL"));
        assert!(!output.contains("Bash Scripts"));
        assert!(!output.contains("Validation"));
        assert!(!output.contains("Authorization"));
    }

    #[test]
    fn renders_the_optional_settings_when_present() {
        let output = configuration(
            &ConfigurationView {
                base_url: Some("https://api.example.com"),
                bash: true,
                skip_validation: true,
                authorization_header: Some("Bearer token"),
                ..view()
            },
            120,
            false,
        );

        assert!(output.contains("🔗 Base URL"));
        assert!(output.contains("🐚 Bash Scripts"));
        assert!(output.contains("✓ Enabled"));
        assert!(output.contains("⚠️  Skipped"));
        assert!(output.contains("Bearer token"));
    }

    #[test]
    fn truncates_long_authorization_headers() {
        let header = "Bearer ".to_string() + &"x".repeat(60);
        let output = configuration(
            &ConfigurationView {
                authorization_header: Some(&header),
                ..view()
            },
            200,
            false,
        );

        assert!(output.contains("..."));
        assert!(!output.contains(&header));
    }

    #[test]
    fn renders_every_statistic_row() {
        let output = statistics(
            &OpenApiStats {
                path_item_count: 2,
                operation_count: 4,
                parameter_count: 4,
                request_body_count: 1,
                response_count: 4,
                header_count: 0,
                link_count: 0,
                callback_count: 0,
                schema_count: 15,
            },
            80,
            false,
        );

        assert!(output.contains("┌─📊 OpenAPI Statistics"));
        for label in [
            "📝 Path Items",
            "⚙️  Operations",
            "📝 Parameters",
            "📦 Request Bodies",
            "📋 Responses",
            "🔗 Links",
            "📞 Callbacks",
            "📝 Schemas",
        ] {
            assert!(output.contains(label), "missing {label}");
        }
        assert!(output.contains("15"));
    }

    #[test]
    fn renders_generated_files_with_their_size() {
        let files = vec![
            ScriptFile::new("GetPet.ps1", "x".repeat(300)),
            ScriptFile::new("PostPet.ps1", "x".repeat(2048)),
        ];

        let output = results(&files, Duration::from_millis(185), "/out", 100, false);

        assert!(output.contains("✅ Generation Complete"));
        assert!(output.contains("│ 📄 Files Generated │ 2"));
        assert!(output.contains("185ms"));
        assert!(output.contains("  📝 GetPet.ps1 (300 bytes)"));
        assert!(output.contains("  📝 PostPet.ps1 (2.0 KB)"));
        assert!(output.ends_with("🎉 Done!\n"));
    }

    #[test]
    fn reports_when_nothing_was_generated() {
        let output = results(&[], Duration::from_millis(1), "/out", 100, false);

        assert!(output.contains("⚠️  Generation Complete (No Files)"));
        assert!(!output.contains("Generated Files:"));
    }

    #[test]
    fn formats_file_sizes_by_magnitude() {
        assert_eq!(file_size(512), "512 bytes");
        assert_eq!(file_size(2048), "2.0 KB");
        assert_eq!(file_size(3 * 1024 * 1024), "3.0 MB");
    }

    #[test]
    fn renders_errors_in_red() {
        assert_eq!(error("boom", false), "\nError:\nboom\n");
        assert!(error("boom", true).contains("\u{1b}[31m"));
    }

    #[test]
    fn renders_azure_progress_messages() {
        assert!(azure_started(false).contains("🔐 Acquiring authorization header"));
        assert!(azure_succeeded(false).contains("✅ Successfully acquired access token"));
        assert!(unsupported_version_tips(false).contains("--skip-validation"));
        assert!(diagnostic("Error", "bad", color::RED, false).starts_with("Error:\nbad"));
    }
}
