//! End to end contracts for the `curlgenerator` binary.

use std::{fs, path::PathBuf, process::Command};

fn binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_BIN_EXE_curlgenerator"));
    path.set_extension(std::env::consts::EXE_EXTENSION);

    path
}

fn specification(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/OpenAPI")
        .join(relative)
}

fn output_directory(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("curlgenerator-cli-{name}"));
    let _ = fs::remove_dir_all(&directory);

    directory
}

fn run(arguments: &[&str]) -> (i32, String) {
    let output = Command::new(binary())
        .args(arguments)
        .output()
        .expect("the binary should run");
    let printed = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);

    (output.status.code().unwrap_or(-1), printed)
}

#[test]
fn prints_help_without_any_arguments() {
    let (code, printed) = run(&[]);

    assert_eq!(code, 0);
    assert!(printed.contains("USAGE:"));
    assert!(printed.contains("curlgenerator [URL or input file] [OPTIONS]"));
    assert!(printed.contains("EXAMPLES:"));
    assert!(printed.contains("ARGUMENTS:"));
    assert!(printed.contains("OPTIONS:"));
}

#[test]
fn prints_help_for_the_help_flag() {
    for flag in ["-h", "--help"] {
        let (code, printed) = run(&[flag]);

        assert_eq!(code, 0);
        assert!(printed.contains("USAGE:"), "{flag} did not print usage");
    }
}

#[test]
fn documents_every_option_in_help() {
    let (_, printed) = run(&["--help"]);

    for option in [
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
        assert!(printed.contains(option), "help is missing {option}");
    }
}

#[test]
fn prints_the_version() {
    for flag in ["-v", "--version"] {
        let (code, printed) = run(&[flag]);

        assert_eq!(code, 0);
        assert_eq!(printed.trim(), env!("CARGO_PKG_VERSION"));
    }
}

#[test]
fn generates_powershell_scripts_from_a_local_specification() {
    let directory = output_directory("powershell");
    let (code, printed) = run(&[
        &specification("v3.0/petstore.json").to_string_lossy(),
        "--output",
        &directory.to_string_lossy(),
        "--no-logging",
    ]);

    assert_eq!(code, 0);
    assert!(printed.contains("🎉 Done!"));
    assert!(directory.join("GetPetById.ps1").exists());
    assert!(
        fs::read_to_string(directory.join("GetPetById.ps1"))
            .unwrap()
            .contains("curl -X GET")
    );
}

#[test]
fn generates_bash_scripts_from_a_local_specification() {
    let directory = output_directory("bash");
    let (code, _) = run(&[
        &specification("v3.0/petstore.json").to_string_lossy(),
        "--output",
        &directory.to_string_lossy(),
        "--bash",
        "--no-logging",
    ]);

    assert_eq!(code, 0);
    assert!(directory.join("GetPetById.sh").exists());
}

#[test]
fn applies_the_authorization_header_to_every_request() {
    let directory = output_directory("authorization");
    let (code, _) = run(&[
        &specification("v3.0/petstore.json").to_string_lossy(),
        "--output",
        &directory.to_string_lossy(),
        "--authorization-header",
        "Bearer token",
        "--no-logging",
    ]);

    assert_eq!(code, 0);
    assert!(
        fs::read_to_string(directory.join("GetPetById.ps1"))
            .unwrap()
            .contains("Authorization: Bearer token")
    );
}

#[test]
fn fails_for_a_missing_specification() {
    let (code, printed) = run(&["./does-not-exist.json", "--no-logging"]);

    assert_eq!(code, 1);
    assert!(printed.contains("Error:"));
}

#[test]
fn fails_for_a_specification_that_does_not_validate() {
    let (code, printed) = run(&[
        &specification("v3.1/non-oauth-scopes.json").to_string_lossy(),
        "--no-logging",
    ]);

    assert_eq!(code, 1);
    assert!(printed.contains("OpenAPI validation failed"));
}

#[test]
fn generates_an_unvalidated_specification_when_validation_is_skipped() {
    let directory = output_directory("skip-validation");
    let (code, printed) = run(&[
        &specification("v3.1/non-oauth-scopes.json").to_string_lossy(),
        "--output",
        &directory.to_string_lossy(),
        "--skip-validation",
        "--no-logging",
    ]);

    assert_eq!(code, 0);
    assert!(printed.contains("🎉 Done!"));
}

/// Requires network access; run with `cargo test -- --ignored`.
#[test]
#[ignore]
fn generates_from_a_remote_specification() {
    let directory = output_directory("remote");
    let (code, printed) = run(&[
        "https://petstore3.swagger.io/api/v3/openapi.json",
        "--output",
        &directory.to_string_lossy(),
        "--no-logging",
        "--skip-validation",
    ]);

    assert_eq!(code, 0);
    assert!(printed.contains("🎉 Done!"));
    assert!(
        fs::read_dir(&directory)
            .expect("the output directory should exist")
            .count()
            > 0
    );
}

#[test]
fn colors_piped_output_only_when_forced() {
    for (environment, colored) in [
        (&[][..], false),
        (&[("CLICOLOR_FORCE", "1")][..], true),
        (&[("CLICOLOR_FORCE", "0")][..], false),
        (
            &[
                ("CLICOLOR_FORCE", "1"),
                ("NO_COLOR", "1"),
                ("CLICOLOR", "0"),
                ("TERM", "dumb"),
            ][..],
            true,
        ),
    ] {
        let output = Command::new(binary())
            .args(["./does-not-exist.json", "--no-logging"])
            .env_remove("CLICOLOR")
            .env_remove("CLICOLOR_FORCE")
            .env_remove("NO_COLOR")
            .envs(environment.iter().copied())
            .output()
            .expect("the binary should run");

        assert_eq!(
            String::from_utf8_lossy(&output.stdout).contains('\u{1b}'),
            colored,
            "unexpected colors with {environment:?}"
        );
    }
}

#[test]
fn lays_out_piped_output_for_80_columns() {
    let help = |columns: Option<&str>| {
        let mut command = Command::new(binary());
        command.arg("--help").env_remove("COLUMNS");
        if let Some(columns) = columns {
            command.env("COLUMNS", columns);
        }

        String::from_utf8_lossy(&command.output().expect("the binary should run").stdout)
            .to_string()
    };

    assert_eq!(help(None), help(Some("80")));
    assert_ne!(help(None), help(Some("120")));
}

#[test]
fn rejects_invalid_arguments_with_a_usage_error() {
    let output = Command::new(binary())
        .args(["./openapi.json", "--unknown"])
        .output()
        .expect("the binary should run");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--unknown"));
}
