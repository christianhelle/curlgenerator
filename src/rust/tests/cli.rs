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

/// Serves HTTP responses on a local port and returns the base URL to reach them.
///
/// `respond` maps a request path to a status line, extra header lines, and a body.
fn serve(respond: fn(&str) -> (&'static str, String, Vec<u8>)) -> String {
    use std::io::{BufRead, Write};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("a local port should be free");
    let address = listener
        .local_addr()
        .expect("the listener should have an address");

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = std::io::BufReader::new(stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }

            let mut header = String::new();
            while reader.read_line(&mut header).is_ok_and(|read| read > 2) {
                header.clear();
            }

            let path = request_line.split_whitespace().nth(1).unwrap_or("/");
            let (status, headers, body) = respond(path);
            let mut stream = reader.into_inner();
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });

    format!("http://{address}")
}

fn petstore(path: &str) -> (&'static str, String, Vec<u8>) {
    let specification =
        fs::read(specification("v3.0/petstore.json")).expect("the petstore specification exists");

    match path {
        "/openapi.json" => ("200 OK", String::new(), specification),
        "/moved" => (
            "302 Found",
            "Location: /openapi.json\r\n".to_string(),
            Vec::new(),
        ),
        "/large.json" => {
            // Trailing whitespace keeps the document valid while pushing it past ten megabytes.
            let mut large = specification;
            large.resize(large.len() + 11 * 1024 * 1024, b' ');
            ("200 OK", String::new(), large)
        }
        _ => ("404 Not Found", String::new(), b"not found".to_vec()),
    }
}

#[test]
fn generates_from_a_specification_served_over_http() {
    let base = serve(petstore);

    for (name, path) in [
        ("http", "/openapi.json"),
        ("http-redirect", "/moved"),
        ("http-large", "/large.json"),
    ] {
        let directory = output_directory(name);
        let (code, printed) = run(&[
            &format!("{base}{path}"),
            "--output",
            &directory.to_string_lossy(),
            "--no-logging",
        ]);

        assert_eq!(code, 0, "{path} failed: {printed}");
        assert!(
            directory.join("GetPetById.ps1").exists(),
            "{path} generated nothing"
        );
    }
}

#[test]
fn fails_for_a_specification_the_server_does_not_have() {
    let (code, printed) = run(&[&format!("{}/missing.json", serve(petstore)), "--no-logging"]);

    assert_eq!(code, 1);
    assert!(
        printed.contains("could not download the file at"),
        "{printed}"
    );
}

/// A fake `az` that records its arguments and prints an access token.
#[cfg(unix)]
const AZ_WITH_TOKEN: &str = r#"echo "$@" > "$(dirname "$0")/az.args"
echo '{"accessToken":"fake-az-token","expiresOn":"2030-01-02 03:04:05.000000","expires_on":1893553445,"tokenType":"Bearer"}'"#;

/// A fake `azd` that records its arguments and prints an access token.
#[cfg(unix)]
const AZD_WITH_TOKEN: &str = r#"echo "$@" > "$(dirname "$0")/azd.args"
echo '{"token":"fake-azd-token","expiresOn":"2030-01-02T03:04:05Z"}'"#;

/// A fake `az` that is not logged in.
#[cfg(unix)]
const AZ_NOT_LOGGED_IN: &str = "echo 'ERROR: Please run az login to setup account.' >&2\nexit 1";

/// Writes fake command line tools into a fresh directory that can be put on `PATH`.
#[cfg(unix)]
fn fake_tools(name: &str, tools: &[(&str, &str)]) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let directory = output_directory(&format!("tools-{name}"));
    fs::create_dir_all(&directory).expect("the tools directory should be created");

    for (tool, script) in tools {
        let path = directory.join(tool);
        fs::write(&path, format!("#!/bin/sh\n{script}\n")).expect("the tool should be written");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .expect("the tool should be executable");
    }

    directory
}

/// Runs the binary against the petstore specification with the fake tools first on `PATH`.
#[cfg(unix)]
fn generate_with_tools(
    tools: &std::path::Path,
    name: &str,
    extra: &[&str],
) -> (i32, String, String) {
    let directory = output_directory(name);
    let output = Command::new(binary())
        .arg(specification("v3.0/petstore.json"))
        .arg("--output")
        .arg(&directory)
        .arg("--no-logging")
        .args(extra)
        .env(
            "PATH",
            format!(
                "{}:{}",
                tools.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("the binary should run");
    let printed = String::from_utf8_lossy(&output.stdout).to_string()
        + &String::from_utf8_lossy(&output.stderr);
    let script = fs::read_to_string(directory.join("GetPetById.ps1")).unwrap_or_default();

    (output.status.code().unwrap_or(-1), printed, script)
}

#[cfg(unix)]
#[test]
fn acquires_the_authorization_header_from_the_azure_cli() {
    let tools = fake_tools("azure-cli", &[("az", AZ_WITH_TOKEN)]);

    let (code, printed, script) = generate_with_tools(
        &tools,
        "azure-cli",
        &[
            "--azure-scope",
            "api://curlgenerator/.default",
            "--azure-tenant-id",
            "my-tenant",
        ],
    );

    assert_eq!(code, 0, "{printed}");
    assert!(
        script.contains("Authorization: Bearer fake-az-token"),
        "{script}"
    );

    let arguments = fs::read_to_string(tools.join("az.args")).expect("az should have run");
    for expected in [
        "account get-access-token",
        "--scope api://curlgenerator/.default",
        "--tenant my-tenant",
    ] {
        assert!(arguments.contains(expected), "{arguments}");
    }
}

#[cfg(unix)]
#[test]
fn falls_back_to_the_azure_developer_cli() {
    let tools = fake_tools(
        "azure-developer-cli",
        &[("az", AZ_NOT_LOGGED_IN), ("azd", AZD_WITH_TOKEN)],
    );

    let (code, printed, script) = generate_with_tools(
        &tools,
        "azure-developer-cli",
        &["--azure-scope", "api://curlgenerator/.default"],
    );

    assert_eq!(code, 0, "{printed}");
    assert!(
        script.contains("Authorization: Bearer fake-azd-token"),
        "{script}"
    );

    let arguments = fs::read_to_string(tools.join("azd.args")).expect("azd should have run");
    for expected in ["auth token", "--scope api://curlgenerator/.default"] {
        assert!(arguments.contains(expected), "{arguments}");
    }
}

#[cfg(unix)]
#[test]
fn keeps_generating_when_no_azure_token_can_be_acquired() {
    let tools = fake_tools(
        "azure-failure",
        &[
            ("az", AZ_NOT_LOGGED_IN),
            ("azd", "echo 'ERROR: not logged in' >&2\nexit 1"),
        ],
    );

    let (code, printed, script) = generate_with_tools(
        &tools,
        "azure-failure",
        &["--azure-scope", "api://curlgenerator/.default"],
    );

    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("Error:"), "{printed}");
    assert!(
        printed.contains("Please run az login to setup account."),
        "{printed}"
    );
    assert!(!script.contains("Authorization:"), "{script}");
}

#[cfg(unix)]
#[test]
fn never_passes_an_invalid_azure_scope_to_the_tools() {
    let tools = fake_tools(
        "azure-invalid-scope",
        &[("az", AZ_WITH_TOKEN), ("azd", AZD_WITH_TOKEN)],
    );

    let (code, printed, script) = generate_with_tools(
        &tools,
        "azure-invalid-scope",
        &[
            "--azure-scope",
            "api://curlgenerator/.default;touch injected",
        ],
    );

    assert_eq!(code, 0, "{printed}");
    assert!(printed.contains("invalid scope"), "{printed}");
    assert!(!tools.join("az.args").exists(), "az should not have run");
    assert!(!tools.join("azd.args").exists(), "azd should not have run");
    assert!(!script.contains("Authorization:"), "{script}");
}
