//! Orchestration of a single generation run.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use curlgenerator_core::{
    GeneratorSettings, ScriptFile,
    generator::generate_from_document,
    openapi::{inspect, load_document, normalize},
};

use crate::{
    args::Args,
    auth::{self, AzureAuth},
    telemetry::Telemetry,
    ui::render::{self, ConfigurationView, color},
    validation,
};

/// Everything a run needs to know about its output destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The terminal width used for layout.
    pub width: usize,
    /// Whether ANSI colors should be emitted.
    pub colors: bool,
}

impl Output {
    /// Detects the terminal capabilities of the current process.
    pub fn detect() -> Self {
        Self {
            width: crate::help::terminal_width(),
            colors: console::colors_enabled(),
        }
    }
}

/// Runs the generator and returns the process exit code.
pub fn run(args: &Args, output: &Output, writer: &mut impl Write) -> std::io::Result<i32> {
    let mut telemetry = Telemetry::new(args.no_logging);
    let code = execute(args, output, writer, &mut telemetry)?;

    pollster::block_on(telemetry.flush());

    Ok(code)
}

fn execute(
    args: &Args,
    output: &Output,
    writer: &mut impl Write,
    telemetry: &mut Telemetry,
) -> std::io::Result<i32> {
    let started = Instant::now();
    let open_api_path = args.open_api_path.clone().unwrap_or_default();

    write!(
        writer,
        "{}",
        render::header(
            env!("CARGO_PKG_VERSION"),
            args.no_logging,
            output.width,
            output.colors
        )
    )?;
    write!(
        writer,
        "{}",
        render::configuration(
            &configuration_view(args, &open_api_path),
            output.width,
            output.colors
        )
    )?;

    let document = match load_document(&open_api_path) {
        Ok(document) => document,
        Err(error) => {
            write!(
                writer,
                "{}",
                render::error(&error.to_string(), output.colors)
            )?;
            telemetry.record_error(&error.to_string(), args);
            return Ok(1);
        }
    };

    if !args.skip_validation {
        let diagnostics = validation::validate(&document);
        if !diagnostics.is_empty() {
            write!(
                writer,
                "{}",
                render::diagnostic(
                    "OpenAPI validation failed",
                    &diagnostics
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n"),
                    color::RED,
                    output.colors
                )
            )?;
            write!(
                writer,
                "{}",
                render::unsupported_version_tips(output.colors)
            )?;
            telemetry.record_error("OpenAPI validation failed", args);
            return Ok(1);
        }

        write!(
            writer,
            "{}",
            render::statistics(&inspect(&document), output.width, output.colors)
        )?;
    }

    let authorization_header = resolve_authorization_header(args, output, writer)?;

    let settings = GeneratorSettings {
        open_api_path: open_api_path.clone(),
        authorization_header,
        content_type: args.content_type.clone(),
        base_url: args.base_url.clone(),
        generate_bash_scripts: args.bash,
    };

    telemetry.record_feature_usage(args);

    let result = generate_from_document(&settings, &normalize(&document));
    write_files(&result.files, &args.output)?;

    write!(
        writer,
        "{}",
        render::results(
            &result.files,
            started.elapsed(),
            &full_path(&args.output),
            output.width,
            output.colors
        )
    )?;

    Ok(0)
}

/// Writes every generated script into the output directory, creating it when needed.
pub fn write_files(files: &[ScriptFile], output: &str) -> std::io::Result<()> {
    let directory = Path::new(output);
    if !output.trim().is_empty() && !directory.exists() {
        fs::create_dir_all(directory)?;
    }

    for file in files {
        fs::write(directory.join(&file.filename), &file.content)?;
    }

    Ok(())
}

/// Returns the authorization header to apply, requesting an Azure Entra ID token when configured.
fn resolve_authorization_header(
    args: &Args,
    output: &Output,
    writer: &mut impl Write,
) -> std::io::Result<Option<String>> {
    if !auth::wants_azure_token(
        args.authorization_header.as_deref(),
        args.azure_scope.as_deref(),
        args.azure_tenant_id.as_deref(),
    ) {
        return Ok(args.authorization_header.clone());
    }

    write!(writer, "{}", render::azure_started(output.colors))?;

    let scope = args.azure_scope.clone().unwrap_or_default();
    match pollster::block_on(auth::acquire_token(&scope, args.azure_tenant_id.as_deref())) {
        AzureAuth::Acquired(header) => {
            write!(writer, "{}", render::azure_succeeded(output.colors))?;
            Ok(Some(header))
        }
        AzureAuth::Failed(reason) => {
            write!(writer, "{}", render::error(&reason, output.colors))?;
            Ok(args.authorization_header.clone())
        }
        AzureAuth::NotRequested => Ok(args.authorization_header.clone()),
    }
}

fn configuration_view<'a>(args: &'a Args, open_api_path: &'a str) -> ConfigurationView<'a> {
    ConfigurationView {
        open_api_path,
        output: &args.output,
        content_type: &args.content_type,
        base_url: args.base_url.as_deref(),
        bash: args.bash,
        skip_validation: args.skip_validation,
        authorization_header: args.authorization_header.as_deref(),
    }
}

/// Returns the absolute output location, keeping a trailing separator when the caller wrote one.
fn full_path(output: &str) -> String {
    let absolute = fs::canonicalize(output)
        .map(|path| {
            path.to_string_lossy()
                .trim_start_matches(r"\\?\")
                .to_string()
        })
        .unwrap_or_else(|_| PathBuf::from(output).to_string_lossy().to_string());

    if output.ends_with(['/', '\\']) && !absolute.ends_with(std::path::MAIN_SEPARATOR) {
        return format!("{absolute}{}", std::path::MAIN_SEPARATOR);
    }

    absolute
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn plain_output() -> Output {
        Output {
            width: 100,
            colors: false,
        }
    }

    fn args_for(spec: &str, output: &Path, extra: &[&str]) -> Args {
        let mut arguments = vec![
            "curlgenerator".to_string(),
            spec.to_string(),
            "--output".to_string(),
            output.to_string_lossy().to_string(),
            "--no-logging".to_string(),
        ];
        arguments.extend(extra.iter().map(ToString::to_string));

        Args::try_parse_from(arguments).expect("arguments should parse")
    }

    fn temp_directory(name: &str) -> PathBuf {
        let directory = std::env::temp_dir().join(format!("curlgenerator-{name}"));
        let _ = fs::remove_dir_all(&directory);

        directory
    }

    fn spec_path(relative: &str) -> String {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../test/OpenAPI")
            .join(relative)
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn generates_scripts_and_reports_success() {
        let directory = temp_directory("run-success");
        let args = args_for(&spec_path("v3.0/petstore-expanded.json"), &directory, &[]);
        let mut buffer = Vec::new();

        let code = run(&args, &plain_output(), &mut buffer).unwrap();
        let printed = String::from_utf8(buffer).unwrap();

        assert_eq!(code, 0);
        assert!(printed.contains("🔧 cURL Request Generator"));
        assert!(printed.contains("📊 OpenAPI Statistics"));
        assert!(printed.contains("✅ Generation Complete"));
        assert!(printed.contains("🎉 Done!"));
        assert!(directory.join("PostAddPet.ps1").exists());
        assert!(directory.join("GetFindPets.ps1").exists());
    }

    #[test]
    fn generates_bash_scripts_when_asked() {
        let directory = temp_directory("run-bash");
        let args = args_for(
            &spec_path("v3.0/petstore-expanded.json"),
            &directory,
            &["--bash"],
        );
        let mut buffer = Vec::new();

        assert_eq!(run(&args, &plain_output(), &mut buffer).unwrap(), 0);
        assert!(directory.join("PostAddPet.sh").exists());
        assert!(
            String::from_utf8(buffer)
                .unwrap()
                .contains("🐚 Bash Scripts")
        );
    }

    #[test]
    fn reports_a_missing_specification() {
        let directory = temp_directory("run-missing");
        let args = args_for("./does-not-exist.json", &directory, &[]);
        let mut buffer = Vec::new();

        let code = run(&args, &plain_output(), &mut buffer).unwrap();

        assert_eq!(code, 1);
        assert!(String::from_utf8(buffer).unwrap().contains("Error:"));
    }

    #[test]
    fn reports_validation_failures_and_skips_generation() {
        let directory = temp_directory("run-invalid");
        let args = args_for(&spec_path("v3.1/non-oauth-scopes.json"), &directory, &[]);
        let mut buffer = Vec::new();

        let code = run(&args, &plain_output(), &mut buffer).unwrap();
        let printed = String::from_utf8(buffer).unwrap();

        assert_eq!(code, 1);
        assert!(printed.contains("OpenAPI validation failed"));
        assert!(printed.contains("Responses must contain at least one response"));
        assert!(printed.contains("--skip-validation"));
        assert!(!printed.contains("🎉 Done!"));
    }

    #[test]
    fn generates_invalid_specifications_when_validation_is_skipped() {
        let directory = temp_directory("run-skip-validation");
        let args = args_for(
            &spec_path("v3.1/non-oauth-scopes.json"),
            &directory,
            &["--skip-validation"],
        );
        let mut buffer = Vec::new();

        let code = run(&args, &plain_output(), &mut buffer).unwrap();
        let printed = String::from_utf8(buffer).unwrap();

        assert_eq!(code, 0);
        assert!(printed.contains("⚠️  Skipped"));
        assert!(!printed.contains("📊 OpenAPI Statistics"));
        assert!(printed.contains("🎉 Done!"));
    }

    #[test]
    fn creates_the_output_directory_when_it_does_not_exist() {
        let directory = temp_directory("run-nested").join("a").join("b");
        let args = args_for(&spec_path("v3.0/petstore-expanded.json"), &directory, &[]);
        let mut buffer = Vec::new();

        assert_eq!(run(&args, &plain_output(), &mut buffer).unwrap(), 0);
        assert!(directory.join("PostAddPet.ps1").exists());
    }
}
