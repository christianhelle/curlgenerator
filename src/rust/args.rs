//! Command line arguments, mirroring the options of the legacy .NET CLI.

use clap::{ArgAction, Parser};

/// The default output directory.
pub const DEFAULT_OUTPUT: &str = "./";

/// The default `Content-Type` header applied to generated requests.
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// Parsed command line arguments.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "curlgenerator",
    version,
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Args {
    /// URL or file path to OpenAPI Specification file.
    #[arg(value_name = "URL or input file")]
    pub open_api_path: Option<String>,

    /// Output directory.
    #[arg(short = 'o', long = "output", value_name = "OUTPUT", default_value = DEFAULT_OUTPUT)]
    pub output: String,

    /// Generate Bash scripts.
    #[arg(long = "bash", action = ArgAction::SetTrue)]
    pub bash: bool,

    /// Don't log errors or collect telemetry.
    #[arg(long = "no-logging", action = ArgAction::SetTrue)]
    pub no_logging: bool,

    /// Skip validation of OpenAPI Specification file.
    #[arg(long = "skip-validation", action = ArgAction::SetTrue)]
    pub skip_validation: bool,

    /// Authorization header to use for all requests.
    #[arg(long = "authorization-header", value_name = "HEADER")]
    pub authorization_header: Option<String>,

    /// Default Content-Type header to use for all requests.
    #[arg(long = "content-type", value_name = "CONTENT-TYPE", default_value = DEFAULT_CONTENT_TYPE)]
    pub content_type: String,

    /// Default Base URL to use for all requests.
    #[arg(long = "base-url", value_name = "BASE-URL")]
    pub base_url: Option<String>,

    /// Azure Entra ID Scope to use for retrieving an access token.
    #[arg(long = "azure-scope", value_name = "SCOPE")]
    pub azure_scope: Option<String>,

    /// Azure Entra ID Tenant ID to use for retrieving an access token.
    #[arg(long = "azure-tenant-id", value_name = "TENANT-ID")]
    pub azure_tenant_id: Option<String>,

    /// Prints help information.
    #[arg(short = 'h', long = "help", action = ArgAction::SetTrue)]
    pub help: bool,

    /// Prints version information.
    #[arg(short = 'v', long = "version", action = ArgAction::SetTrue)]
    pub version: bool,
}

impl Args {
    /// Returns `true` when the invocation only asks for help.
    ///
    /// An invocation without any arguments prints help, matching the legacy CLI.
    pub fn wants_help(&self) -> bool {
        self.help || self.open_api_path.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Args {
        let mut all = vec!["curlgenerator"];
        all.extend_from_slice(arguments);

        Args::try_parse_from(all).expect("arguments should parse")
    }

    #[test]
    fn applies_the_documented_defaults() {
        let args = parse(&["./openapi.json"]);

        assert_eq!(args.open_api_path.as_deref(), Some("./openapi.json"));
        assert_eq!(args.output, "./");
        assert_eq!(args.content_type, "application/json");
        assert!(!args.bash);
        assert!(!args.no_logging);
        assert!(!args.skip_validation);
        assert_eq!(args.authorization_header, None);
        assert_eq!(args.base_url, None);
        assert_eq!(args.azure_scope, None);
        assert_eq!(args.azure_tenant_id, None);
        assert!(!args.wants_help());
    }

    #[test]
    fn parses_every_option() {
        let args = parse(&[
            "https://example.com/openapi.json",
            "--output",
            "./out",
            "--bash",
            "--no-logging",
            "--skip-validation",
            "--authorization-header",
            "Bearer token",
            "--content-type",
            "application/xml",
            "--base-url",
            "https://api.example.com",
            "--azure-scope",
            "api://scope/.default",
            "--azure-tenant-id",
            "tenant",
        ]);

        assert_eq!(args.output, "./out");
        assert!(args.bash);
        assert!(args.no_logging);
        assert!(args.skip_validation);
        assert_eq!(args.authorization_header.as_deref(), Some("Bearer token"));
        assert_eq!(args.content_type, "application/xml");
        assert_eq!(args.base_url.as_deref(), Some("https://api.example.com"));
        assert_eq!(args.azure_scope.as_deref(), Some("api://scope/.default"));
        assert_eq!(args.azure_tenant_id.as_deref(), Some("tenant"));
    }

    #[test]
    fn accepts_the_short_output_option() {
        assert_eq!(parse(&["./openapi.json", "-o", "./out"]).output, "./out");
    }

    #[test]
    fn asks_for_help_without_arguments_or_with_the_help_flag() {
        assert!(parse(&[]).wants_help());
        assert!(parse(&["--help"]).wants_help());
        assert!(parse(&["-h"]).wants_help());
        assert!(parse(&["./openapi.json", "--help"]).wants_help());
    }

    #[test]
    fn recognizes_the_version_flag() {
        assert!(parse(&["--version"]).version);
        assert!(parse(&["-v"]).version);
    }
}
