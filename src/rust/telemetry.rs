//! Anonymous feature usage and error reporting.
//!
//! Reporting is disabled entirely by `--no-logging`. Nothing is sent while events are recorded;
//! they are buffered and submitted once at the end of a run.

use crate::{anonymous_identity, support_key};
use exceptionless::ExceptionlessClient;
use serde_json::json;

use crate::{args::Args, privacy::redact_authorization_headers};

/// The Exceptionless project the legacy .NET CLI reports to.
const API_KEY: &str = "0uYLSLp8xgVp1t5euguXwrmvb5JieO3uE0N1VgwT";

/// A recorded telemetry event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A command line option was used.
    FeatureUsage {
        /// The long option name.
        feature: String,
    },
    /// The run failed.
    Error {
        /// The failure description.
        message: String,
        /// The redacted command line.
        command_line: String,
        /// The redacted settings, serialized as JSON.
        settings: String,
    },
}

/// Buffers telemetry events until they are submitted.
#[derive(Debug, Default)]
pub struct Telemetry {
    enabled: bool,
    events: Vec<Event>,
}

impl Telemetry {
    /// Creates a recorder that is disabled when `--no-logging` was passed.
    pub fn new(no_logging: bool) -> Self {
        Self {
            enabled: !no_logging,
            events: Vec::new(),
        }
    }

    /// Returns `true` when events will be reported.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the recorded events.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Records one feature usage event per option the invocation actually used.
    ///
    /// The output directory and the logging switch are never reported, matching the legacy CLI.
    pub fn record_feature_usage(&mut self, args: &Args) {
        if !self.enabled {
            return;
        }

        for feature in used_features(args) {
            self.events.push(Event::FeatureUsage {
                feature: feature.to_string(),
            });
        }
    }

    /// Records a failed run together with the redacted invocation that produced it.
    pub fn record_error(&mut self, message: &str, args: &Args) {
        if !self.enabled {
            return;
        }

        self.events.push(Event::Error {
            message: message.to_string(),
            command_line: redact_authorization_headers(&command_line()),
            settings: redacted_settings(args),
        });
    }

    /// Submits every recorded event and clears the buffer.
    pub async fn flush(&mut self) {
        if !self.enabled || self.events.is_empty() {
            return;
        }

        let client = ExceptionlessClient::with_api_key(API_KEY);
        let identity = anonymous_identity();
        let key = support_key();

        for event in std::mem::take(&mut self.events) {
            let _ = match event {
                Event::FeatureUsage { feature } => {
                    client
                        .feature(&feature)
                        .user_identity(&identity)
                        .data("supportKey", key.as_str())
                        .send()
                        .await
                }
                Event::Error {
                    message,
                    command_line,
                    settings,
                } => {
                    client
                        .error(&GenerationError(message))
                        .tag("error")
                        .source("curlgenerator")
                        .user_identity(&identity)
                        .data("supportKey", key.as_str())
                        .data("commandLine", command_line.as_str())
                        .data("settings", serde_json::Value::String(settings))
                        .send()
                        .await
                }
            };
        }
    }
}

/// Returns the long option names the invocation used.
///
/// # Examples
///
/// ```
/// use clap::Parser;
/// use curlgenerator::{args::Args, telemetry::used_features};
///
/// let args = Args::try_parse_from(["curlgenerator", "./openapi.json", "--bash"]).unwrap();
///
/// assert_eq!(used_features(&args), vec!["bash", "content-type"]);
/// ```
pub fn used_features(args: &Args) -> Vec<&'static str> {
    let mut features = Vec::new();

    if args.bash {
        features.push("bash");
    }

    if args.skip_validation {
        features.push("skip-validation");
    }

    if args.authorization_header.is_some() {
        features.push("authorization-header");
    }

    if !args.content_type.is_empty() {
        features.push("content-type");
    }

    if args.base_url.is_some() {
        features.push("base-url");
    }

    if args.azure_scope.is_some() {
        features.push("azure-scope");
    }

    if args.azure_tenant_id.is_some() {
        features.push("azure-tenant-id");
    }

    features
}

fn redacted_settings(args: &Args) -> String {
    json!({
        "openApiPath": args.open_api_path,
        "outputFolder": args.output,
        "generateBashScripts": args.bash,
        "skipValidation": args.skip_validation,
        "contentType": args.content_type,
        "baseUrl": args.base_url,
        "authorizationHeader": args.authorization_header.as_ref().map(|_| "[REDACTED]"),
        "azureScope": args.azure_scope,
        "azureTenantId": args.azure_tenant_id,
    })
    .to_string()
}

fn command_line() -> String {
    std::env::args().collect::<Vec<_>>().join(" ")
}

#[derive(Debug)]
struct GenerationError(String);

impl std::fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for GenerationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(arguments: &[&str]) -> Args {
        let mut all = vec!["curlgenerator"];
        all.extend_from_slice(arguments);

        Args::try_parse_from(all).expect("arguments should parse")
    }

    #[test]
    fn records_nothing_when_logging_is_disabled() {
        let args = parse(&["./openapi.json", "--bash", "--no-logging"]);
        let mut telemetry = Telemetry::new(args.no_logging);

        telemetry.record_feature_usage(&args);
        telemetry.record_error("boom", &args);

        assert!(!telemetry.is_enabled());
        assert!(telemetry.events().is_empty());
    }

    #[test]
    fn records_one_event_per_used_option() {
        let args = parse(&["./openapi.json", "--bash", "--skip-validation"]);
        let mut telemetry = Telemetry::new(args.no_logging);

        telemetry.record_feature_usage(&args);

        assert_eq!(
            telemetry.events(),
            [
                Event::FeatureUsage {
                    feature: "bash".to_string()
                },
                Event::FeatureUsage {
                    feature: "skip-validation".to_string()
                },
                Event::FeatureUsage {
                    feature: "content-type".to_string()
                },
            ]
        );
    }

    #[test]
    fn never_reports_the_output_directory_or_the_logging_switch() {
        let features = used_features(&parse(&["./openapi.json", "--output", "./out"]));

        assert!(!features.contains(&"output"));
        assert!(!features.contains(&"no-logging"));
    }

    #[test]
    fn reports_every_optional_value_that_was_supplied() {
        let features = used_features(&parse(&[
            "./openapi.json",
            "--authorization-header",
            "Bearer token",
            "--base-url",
            "https://example.com",
            "--azure-scope",
            "scope",
            "--azure-tenant-id",
            "tenant",
        ]));

        assert_eq!(
            features,
            vec![
                "authorization-header",
                "content-type",
                "base-url",
                "azure-scope",
                "azure-tenant-id"
            ]
        );
    }

    #[test]
    fn redacts_the_authorization_header_from_reported_settings() {
        let args = parse(&["./openapi.json", "--authorization-header", "Bearer secret"]);
        let mut telemetry = Telemetry::new(args.no_logging);

        telemetry.record_error("boom", &args);

        let Event::Error {
            message, settings, ..
        } = &telemetry.events()[0]
        else {
            panic!("expected an error event");
        };

        assert_eq!(message, "boom");
        assert!(settings.contains("[REDACTED]"));
        assert!(!settings.contains("secret"));
    }
}
