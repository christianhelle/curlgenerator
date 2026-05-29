//! Command line entry point for the cURL Request Generator.

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use owo_colors::{OwoColorize, Style};

use curlgenerator::generator::{self, GeneratorResult, GeneratorSettings};
use curlgenerator::validation::{self, OpenApiStats};
use curlgenerator::{azure, support};

mod console;
use console::{Align, Panel, Table};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Generate cURL requests from OpenAPI specifications (v2.0, v3.0 and v3.1).
#[derive(Debug, Parser)]
#[command(
    name = "curlgenerator",
    version = VERSION,
    about = "Generate cURL requests from OpenAPI specifications (v2.0, v3.0 and v3.1)",
    after_help = "EXAMPLES:\n  \
        curlgenerator ./openapi.json\n  \
        curlgenerator ./openapi.json --output ./\n  \
        curlgenerator ./openapi.json --bash\n  \
        curlgenerator https://petstore.swagger.io/v2/swagger.json\n  \
        curlgenerator https://petstore3.swagger.io/api/v3/openapi.json --base-url https://petstore3.swagger.io\n  \
        curlgenerator ./openapi.json --azure-scope [Some Application ID URI]/.default"
)]
struct Cli {
    /// URL or file path to OpenAPI Specification file.
    #[arg(value_name = "URL or input file")]
    open_api_path: String,

    /// Output directory.
    #[arg(short, long, default_value = "./")]
    output: String,

    /// Generate Bash scripts.
    #[arg(long)]
    bash: bool,

    /// Don't log errors or collect telemetry (accepted for compatibility; no-op).
    #[arg(long)]
    no_logging: bool,

    /// Skip validation of OpenAPI Specification file.
    #[arg(long)]
    skip_validation: bool,

    /// Authorization header to use for all requests.
    #[arg(long, value_name = "HEADER")]
    authorization_header: Option<String>,

    /// Default Content-Type header to use for all requests.
    #[arg(long, value_name = "CONTENT-TYPE", default_value = "application/json")]
    content_type: String,

    /// Default Base URL to use for all requests.
    #[arg(long, value_name = "BASE-URL")]
    base_url: Option<String>,

    /// Azure Entra ID Scope to use for retrieving an Access Token.
    #[arg(long, value_name = "SCOPE")]
    azure_scope: Option<String>,

    /// Azure Entra ID Tenant ID to use for retrieving an Access Token.
    #[arg(long, value_name = "TENANT-ID")]
    azure_tenant_id: Option<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("\n{}", format!("Error:\n{message}").red());
            ExitCode::FAILURE
        }
    }
}

fn run(mut cli: Cli) -> Result<(), String> {
    let start = std::time::Instant::now();

    display_header(&cli);
    display_configuration(&cli);

    if !cli.skip_validation {
        validate_spec(&cli.open_api_path)?;
    }

    acquire_azure_token(&mut cli);

    let settings = GeneratorSettings {
        open_api_path: cli.open_api_path.clone(),
        authorization_header: cli.authorization_header.clone(),
        content_type: cli.content_type.clone(),
        base_url: cli.base_url.clone(),
        generate_bash_scripts: cli.bash,
    };

    let result = generator::generate(&settings)?;

    if !cli.output.trim().is_empty() && !Path::new(&cli.output).exists() {
        std::fs::create_dir_all(&cli.output)
            .map_err(|e| format!("Could not create output directory {}: {e}", cli.output))?;
    }

    for file in &result.files {
        let path = Path::new(&cli.output).join(&file.filename);
        std::fs::write(&path, &file.content)
            .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    }

    display_results(&result, start.elapsed(), &cli);
    Ok(())
}

fn validate_spec(open_api_path: &str) -> Result<(), String> {
    let result = validation::validate(open_api_path)?;

    if !result.is_valid() {
        println!(
            "\n{}",
            "OpenAPI validation failed:".style(Style::new().red())
        );
        for error in &result.errors {
            println!("{}", format!("Error:\n{error}").style(Style::new().red()));
        }
        for warning in &result.warnings {
            println!(
                "{}",
                format!("Warning:\n{warning}").style(Style::new().yellow())
            );
        }
        return Err("OpenAPI validation failed".to_string());
    }

    display_statistics(&result.statistics);
    Ok(())
}

fn acquire_azure_token(cli: &mut Cli) {
    let has_auth = cli
        .authorization_header
        .as_deref()
        .map(|h| !h.trim().is_empty())
        .unwrap_or(false);
    let scope = cli.azure_scope.clone().unwrap_or_default();
    let tenant = cli.azure_tenant_id.clone().unwrap_or_default();

    if has_auth || (scope.trim().is_empty() && tenant.trim().is_empty()) {
        return;
    }

    println!(
        "{}",
        "🔐 Acquiring authorization header from Azure Entra ID...".style(Style::new().green())
    );

    match azure::try_get_access_token(cli.azure_tenant_id.as_deref(), &scope) {
        Ok(Some(token)) => {
            cli.authorization_header = Some(format!("Bearer {token}"));
            println!(
                "{}",
                "✅ Successfully acquired access token".style(Style::new().green())
            );
        }
        Ok(None) => {
            eprintln!(
                "{}",
                "No access token was returned".style(Style::new().yellow())
            );
        }
        Err(error) => {
            eprintln!("{}", format!("Error:\n{error}").style(Style::new().red()));
        }
    }
}

fn display_header(cli: &Cli) {
    let green = Style::new().green();
    Panel::new(green)
        .expand()
        .line(paint_line(
            &format!("🔧 cURL Request Generator v{VERSION}"),
            green.bold(),
        ))
        .print();
    println!();

    if cli.no_logging {
        println!(
            "{}",
            "⚠️  Unavailable when logging is disabled".style(Style::new().yellow())
        );
    } else {
        println!(
            "{}",
            format!("🔑 Support key: {}", support::get_support_key()).style(green)
        );
    }
    println!();
}

fn paint_line(text: &str, style: Style) -> String {
    format!("{}", text.style(style))
}

fn display_configuration(cli: &Cli) {
    let plain = Style::new();
    let cyan = Style::new().cyan();

    let mut table = Table::new(Style::new().bright_black())
        .column("Setting", Align::Left)
        .column("Value", Align::Left)
        .row(vec![
            ("📁 OpenAPI Source".to_string(), plain),
            (cli.open_api_path.clone(), cyan),
        ])
        .row(vec![
            ("📂 Output Folder".to_string(), plain),
            (cli.output.clone(), cyan),
        ])
        .row(vec![
            ("🌐 Content Type".to_string(), plain),
            (cli.content_type.clone(), cyan),
        ]);

    if let Some(base_url) = cli.base_url.as_deref().filter(|s| !s.trim().is_empty()) {
        table = table.row(vec![
            ("🔗 Base URL".to_string(), plain),
            (base_url.to_string(), cyan),
        ]);
    }
    if cli.bash {
        table = table.row(vec![
            ("🐚 Bash Scripts".to_string(), plain),
            ("✓ Enabled".to_string(), Style::new().green()),
        ]);
    }
    if cli.skip_validation {
        table = table.row(vec![
            ("⚠️  Validation".to_string(), plain),
            ("⚠️  Skipped".to_string(), Style::new().yellow()),
        ]);
    }
    if let Some(auth) = cli
        .authorization_header
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        let shown = if auth.len() > 50 {
            format!("{}...", &auth[..47])
        } else {
            auth.to_string()
        };
        table = table.row(vec![
            ("🔐 Authorization".to_string(), plain),
            (shown, Style::new().dimmed()),
        ]);
    }

    Panel::new(Style::new().yellow())
        .title("📋 Configuration", Style::new().yellow().bold())
        .content(table.render())
        .print();
    println!();
}

fn display_statistics(stats: &OpenApiStats) {
    let plain = Style::new();
    let blue = Style::new().blue();
    let table = Table::new(blue)
        .column("Component", Align::Left)
        .column("Count", Align::Right)
        .row(vec![
            ("📝 Path Items".to_string(), plain),
            (stats.path_item_count.to_string(), blue),
        ])
        .row(vec![
            ("⚙️  Operations".to_string(), plain),
            (stats.operation_count.to_string(), blue),
        ])
        .row(vec![
            ("📝 Parameters".to_string(), plain),
            (stats.parameter_count.to_string(), blue),
        ])
        .row(vec![
            ("📦 Request Bodies".to_string(), plain),
            (stats.request_body_count.to_string(), blue),
        ])
        .row(vec![
            ("📋 Responses".to_string(), plain),
            (stats.response_count.to_string(), blue),
        ])
        .row(vec![
            ("🔗 Links".to_string(), plain),
            (stats.link_count.to_string(), blue),
        ])
        .row(vec![
            ("📞 Callbacks".to_string(), plain),
            (stats.callback_count.to_string(), blue),
        ])
        .row(vec![
            ("📝 Schemas".to_string(), plain),
            (stats.schema_count.to_string(), blue),
        ]);

    Panel::new(blue)
        .title("📊 OpenAPI Statistics", blue.bold())
        .content(table.render())
        .print();
    println!();
}

fn display_results(result: &GeneratorResult, elapsed: std::time::Duration, cli: &Cli) {
    let output = Path::new(&cli.output);
    let full_path = output
        .canonicalize()
        .map(|p| {
            let s = p.display().to_string();
            s.strip_prefix(r"\\?\").map(str::to_string).unwrap_or(s)
        })
        .unwrap_or_else(|_| cli.output.clone());

    let plain = Style::new();
    let green = Style::new().green();
    let cyan = Style::new().cyan();

    let table = Table::new(green)
        .column("Metric", Align::Left)
        .column("Value", Align::Left)
        .row(vec![
            ("📄 Files Generated".to_string(), plain),
            (result.files.len().to_string(), green),
        ])
        .row(vec![
            ("⏱️  Duration".to_string(), plain),
            (format!("{}ms", elapsed.as_millis()), green),
        ])
        .row(vec![
            ("📁 Output Location".to_string(), plain),
            (full_path, cyan),
        ]);

    if result.files.is_empty() {
        Panel::new(Style::new().yellow())
            .title(
                "⚠️  Generation Complete (No Files)",
                Style::new().yellow().bold(),
            )
            .content(table.render())
            .print();
    } else {
        Panel::new(green)
            .title("✅ Generation Complete", green.bold())
            .content(table.render())
            .print();

        println!(
            "{}",
            "📁 Generated Files:".style(Style::new().yellow().bold())
        );
        for file in &result.files {
            let size = file.content.len();
            let size_text = if size < 1024 {
                format!("{size} bytes")
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            };
            println!(
                "  📝 {} {}",
                file.filename.style(cyan),
                format!("({size_text})").style(Style::new().dimmed())
            );
        }
    }

    println!();
    println!("{}", "🎉 Done!".style(green.bold()));
}
