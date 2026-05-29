//! Command line entry point for the cURL Request Generator.

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use owo_colors::OwoColorize;

use curlgenerator::generator::{self, GeneratorResult, GeneratorSettings};
use curlgenerator::validation::{self, OpenApiStats};
use curlgenerator::{azure, support};

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
        println!("\n{}", "OpenAPI validation failed:".red());
        for error in &result.errors {
            println!("{}", format!("Error: {error}").red());
        }
        for warning in &result.warnings {
            println!("{}", format!("Warning: {warning}").yellow());
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
        "Acquiring authorization header from Azure Entra ID...".green()
    );

    match azure::try_get_access_token(cli.azure_tenant_id.as_deref(), &scope) {
        Ok(Some(token)) => {
            cli.authorization_header = Some(format!("Bearer {token}"));
            println!("{}", "Successfully acquired access token".green());
        }
        Ok(None) => {
            eprintln!("{}", "No access token was returned".yellow());
        }
        Err(error) => {
            eprintln!("{}", format!("Error:\n{error}").red());
        }
    }
}

fn display_header(cli: &Cli) {
    println!(
        "{}",
        format!("cURL Request Generator v{VERSION}").green().bold()
    );

    if cli.no_logging {
        println!(
            "{}",
            "Support key: unavailable when logging is disabled".yellow()
        );
    } else {
        println!(
            "{}",
            format!("Support key: {}", support::get_support_key()).green()
        );
    }
    println!();
}

fn display_configuration(cli: &Cli) {
    println!("{}", "Configuration".yellow().bold());
    println!("  OpenAPI Source : {}", cli.open_api_path.cyan());
    println!("  Output Folder  : {}", cli.output.cyan());
    println!("  Content Type   : {}", cli.content_type.cyan());

    if let Some(base_url) = cli.base_url.as_deref().filter(|s| !s.trim().is_empty()) {
        println!("  Base URL       : {}", base_url.cyan());
    }
    if cli.bash {
        println!("  Bash Scripts   : {}", "Enabled".green());
    }
    if cli.skip_validation {
        println!("  Validation     : {}", "Skipped".yellow());
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
        println!("  Authorization  : {}", shown.dimmed());
    }
    println!();
}

fn display_statistics(stats: &OpenApiStats) {
    println!("{}", "OpenAPI Statistics".blue().bold());
    println!("  Path Items     : {}", stats.path_item_count.blue());
    println!("  Operations     : {}", stats.operation_count.blue());
    println!("  Parameters     : {}", stats.parameter_count.blue());
    println!("  Request Bodies : {}", stats.request_body_count.blue());
    println!("  Responses      : {}", stats.response_count.blue());
    println!("  Links          : {}", stats.link_count.blue());
    println!("  Callbacks      : {}", stats.callback_count.blue());
    println!("  Schemas        : {}", stats.schema_count.blue());
    println!();
}

fn display_results(result: &GeneratorResult, elapsed: std::time::Duration, cli: &Cli) {
    let output = Path::new(&cli.output);
    let full_path = output
        .canonicalize()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| cli.output.clone());

    println!("{}", "Generation Complete".green().bold());
    println!("  Files Generated: {}", result.files.len().green());
    println!("  Duration       : {}ms", elapsed.as_millis().green());
    println!("  Output Location: {}", full_path.cyan());
    println!();

    if !result.files.is_empty() {
        println!("{}", "Generated Files:".yellow().bold());
        for file in &result.files {
            let size = file.content.len();
            let size_text = if size < 1024 {
                format!("{size} bytes")
            } else {
                format!("{:.1} KB", size as f64 / 1024.0)
            };
            println!("  {} ({})", file.filename.cyan(), size_text.dimmed());
        }
    }

    println!("\n{}", "Done!".green().bold());
}
