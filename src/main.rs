mod cli;
mod generator;
mod models;
mod names;
mod openapi;
mod stats;
mod validation;

use cli::Cli;
use generator::generate;
use models::GeneratorSettings;
use validation::validate;

use clap::Parser;
use console::style;
use std::path::Path;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Default to help if no arguments provided
    if cli.openapi_path.is_empty() {
        Cli::parse_from(["curlgenerator", "--help"]);
        return;
    }

    let start = Instant::now();

    // Display header
    display_header();

    // Display configuration
    display_configuration(&cli);

    // Validate if not skipped
    if !cli.skip_validation {
        match validate(&cli.openapi_path).await {
            Ok(result) => {
                if !result.is_valid {
                    eprintln!("\nOpenAPI validation failed:");
                    for diag in &result.diagnostics {
                        eprintln!("  {}", diag);
                    }
                    std::process::exit(1);
                }
                display_stats(&result.statistics);
            }
            Err(e) => {
                eprintln!("\nError: {}", style(e).red());
                eprintln!("\nTips:");
                eprintln!("  Consider using the --skip-validation argument.");
                eprintln!("  In some cases, the features that are specific to the");
                eprintln!("  unsupported versions of OpenAPI specifications aren't really used.");
                eprintln!("  This tool uses Microsoft.OpenApi libraries for both parsing and validation.");
                std::process::exit(1);
            }
        }
    }

    // Generate scripts
    let settings = GeneratorSettings {
        openapi_path: cli.openapi_path.clone(),
        authorization_header: cli.authorization_header.clone(),
        content_type: cli.content_type.clone(),
        base_url: cli.base_url.clone(),
        generate_bash: cli.bash,
    };

    let result = match generate(&settings).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("\nError: {}", style(e).red());
            std::process::exit(1);
        }
    };

    // Write files
    if !cli.output.is_empty() && !cli.output.contains('.') {
        let output_path = Path::new(&cli.output);
        if !output_path.exists() {
            if let Err(e) = std::fs::create_dir_all(output_path) {
                eprintln!("\nError creating output directory: {}", e);
                std::process::exit(1);
            }
        }
    }

    for file in &result.files {
        let filepath = if cli.output.ends_with('/') || cli.output.ends_with('\\') {
            format!("{}{}", cli.output, file.filename)
        } else if cli.output == "./" || cli.output == "." {
            file.filename.clone()
        } else {
            format!("{}/{}", cli.output, file.filename)
        };

        if let Err(e) = std::fs::write(&filepath, &file.content) {
            eprintln!("\nError writing {}: {}", filepath, e);
            std::process::exit(1);
        }
    }

    let elapsed = start.elapsed();
    display_results(&result, &elapsed);
}

fn display_header() {
    println!(
        "{}",
        style("🔧 cURL Request Generator").green().bold()
    );
    println!();
}

fn display_configuration(cli: &Cli) {
    println!("{}", style("📋 Configuration").yellow().bold());
    println!("  OpenAPI Source: {}", cli.openapi_path);
    println!("  Output Folder: {}", cli.output);
    println!("  Content Type: {}", cli.content_type);

    if let Some(ref base) = cli.base_url {
        println!("  Base URL: {}", base);
    }

    if cli.bash {
        println!("  Bash Scripts: ✓ Enabled");
    }

    if cli.skip_validation {
        println!("  Validation: ⚠️  Skipped");
    }

    if let Some(ref auth) = cli.authorization_header {
        let display = if auth.len() > 50 {
            format!("{}...", &auth[..47])
        } else {
            auth.clone()
        };
        println!("  Authorization: {}", style(display).dim());
    }

    println!();
}

fn display_stats(stats: &stats::OpenApiStats) {
    println!("{}", style("📊 OpenAPI Statistics").blue().bold());
    println!("  Path Items: {}", stats.path_item_count);
    println!("  Operations: {}", stats.operation_count);
    println!("  Parameters: {}", stats.parameter_count);
    println!("  Request Bodies: {}", stats.request_body_count);
    println!("  Responses: {}", stats.response_count);
    println!("  Links: {}", stats.link_count);
    println!("  Callbacks: {}", stats.callback_count);
    println!("  Schemas: {}", stats.schema_count);
    println!();
}

fn display_results(result: &models::GeneratorResult, elapsed: &std::time::Duration) {
    println!("{}", style("✅ Generation Complete").green().bold());
    println!("  Files Generated: {}", result.files.len());
    println!("  Duration: {:.0}ms", elapsed.as_micros() as f64 / 1000.0);
    println!();

    if !result.files.is_empty() {
        println!("{}", style("📁 Generated Files:").yellow().bold());
        for file in &result.files {
            let size = file.content.len();
            let size_str = if size < 1024 {
                format!("{} bytes", size)
            } else if size < 1024 * 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            };
            println!("  {} ({})", file.filename, size_str);
        }
    }

    println!();
    println!("{}", style("🎉 Done!").green().bold());
}
