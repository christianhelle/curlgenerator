mod cli;
mod generator;
mod openapi;
mod script;
mod error;

use anyhow::Result;
use cli::Cli;
use clap::Parser;
use colored::*;
use std::time::Instant;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // If no OpenAPI path provided, show help and exit
    if cli.openapi_path.is_none() {
        Cli::parse_from(["curlgenerator", "--help"]);
        return Ok(());
    }
    
    let openapi_path = cli.openapi_path.as_ref().unwrap();
    
    let start = Instant::now();
    
    display_header();
    display_configuration(&cli);
    
    let document = openapi::load_document(openapi_path)?;
    if !cli.skip_validation {
        display_statistics(&document);
    }
    
    let settings = generator::GeneratorSettings {
        authorization_header: cli.authorization_header.clone(),
        content_type: cli.content_type.clone(),
        base_url: cli.base_url.clone(),
        generate_bash_scripts: cli.bash,
    };
    
    let result = generator::generate(&document, &settings)?;
    
    if !std::path::Path::new(&cli.output).exists() {
        std::fs::create_dir_all(&cli.output)?;
    }
    
    for file in &result.files {
        let path = std::path::Path::new(&cli.output).join(&file.filename);
        std::fs::write(&path, &file.content)?;
    }
    
    let duration = start.elapsed();
    display_results(&result, duration, &cli.output);
    
    Ok(())
}

fn display_header() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{}", "🔧  cURL Request Generator".green().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
}

fn display_configuration(cli: &Cli) {
    println!("{}", "📋  Configuration".yellow().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {} {}", "📁  OpenAPI Source:".bold(), cli.openapi_path.as_ref().unwrap().cyan());
    println!("  {} {}", "📂  Output Folder:".bold(), cli.output.cyan());
    println!("  {} {}", "🌐  Content Type:".bold(), cli.content_type.cyan());
    
    if let Some(base_url) = &cli.base_url {
        println!("  {} {}", "🔗 Base URL:".bold(), base_url.cyan());
    }
    
    if cli.bash {
        println!("  {} {}", "🐚  Bash Scripts:".bold(), "✓ Enabled".green());
    }
    
    if cli.skip_validation {
        println!("  {} {}", "⚠️  Validation:".bold(), "⚠️  Skipped".yellow());
    }
    
    if cli.authorization_header.is_some() {
        println!("  {} {}", "🔐  Authorization:".bold(), "Present".dimmed());
    }
    
    println!();
}

fn display_statistics(document: &openapiv3::OpenAPI) {
    let path_count = document.paths.paths.len();
    let mut operation_count = 0;
    let mut parameter_count = 0;
    
    for (_path, item) in &document.paths.paths {
        if let openapiv3::ReferenceOr::Item(path_item) = item {
            for operation in path_item.iter() {
                operation_count += 1;
                parameter_count += operation.1.parameters.len();
            }
        }
    }
    
    let schema_count = document.components.as_ref()
        .map(|c| c.schemas.len())
        .unwrap_or(0);
    
    println!("{}", "📊  OpenAPI Statistics".blue().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {} {}", "📝  Path Items:".bold(), path_count.to_string().blue());
    println!("  {} {}", "⚙️  Operations:".bold(), operation_count.to_string().blue());
    println!("  {} {}", "📝  Parameters:".bold(), parameter_count.to_string().blue());
    println!("  {} {}", "📝  Schemas:".bold(), schema_count.to_string().blue());
    println!();
}

fn display_results(result: &generator::GeneratorResult, duration: std::time::Duration, output: &str) {
    println!("{}", "✅ Generation Complete".green().bold());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  {} {}", "📄  Files Generated:".bold(), result.files.len().to_string().green());
    println!("  {} {}ms", "⏱️  Duration:".bold(), duration.as_millis().to_string().green());
    
    let full_path = std::fs::canonicalize(output).unwrap_or_else(|_| std::path::PathBuf::from(output));
    println!("  {} {}", "📁  Output Location:".bold(), full_path.display().to_string().cyan());
    println!();
    
    if !result.files.is_empty() {
        println!("{}", "📁  Generated Files:".yellow().bold());
        for file in &result.files {
            let size = file.content.len();
            let size_str = if size < 1024 {
                format!("{} bytes", size)
            } else {
                format!("{:.1} KB", size as f64 / 1024.0)
            };
            println!("  📝  {} {}", file.filename.cyan(), format!("({})", size_str).dimmed());
        }
    }
    
    println!();
    println!("{}", "🎉  Done!".green().bold());
}
