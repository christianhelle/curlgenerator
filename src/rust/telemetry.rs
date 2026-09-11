//! Anonymous feature usage and error reporting.
//!
//! Reporting is disabled entirely by `--no-logging`. Nothing is sent while events are recorded;
//! they are buffered and submitted once at the end of a run.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{anonymous_identity, support_key};
use serde_json::{Value, json};

use crate::{args::Args, privacy::redact_authorization_headers};

/// The Exceptionless project the legacy .NET CLI reports to.
const API_KEY: &str = "0uYLSLp8xgVp1t5euguXwrmvb5JieO3uE0N1VgwT";

/// The Exceptionless endpoint that accepts batches of events.
const COLLECTOR_URL: &str = "https://collector.exceptionless.io/api/v2/events";

/// The error type failed runs are reported with, as the Exceptionless client used to name it.
const ERROR_TYPE: &str = "curlgenerator::telemetry::GenerationError";

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
        self.flush_to(COLLECTOR_URL);
    }

    /// Submits every recorded event to `endpoint` in a single request and clears the buffer.
    ///
    /// Reporting is best effort, so a failed submission is ignored.
    pub fn flush_to(&mut self, endpoint: &str) {
        if !self.enabled || self.events.is_empty() {
            return;
        }

        let identity = anonymous_identity();
        let key = support_key();
        let date = utc_timestamp(SystemTime::now());

        let events: Vec<Value> = std::mem::take(&mut self.events)
            .into_iter()
            .map(|event| match event {
                Event::FeatureUsage { feature } => json!({
                    "type": "usage",
                    "source": feature,
                    "date": date,
                    "data": { "@user": identity, "supportKey": key },
                }),
                Event::Error {
                    message,
                    command_line,
                    settings,
                } => json!({
                    "type": "error",
                    "source": "curlgenerator",
                    "date": date,
                    "tags": ["error"],
                    "data": {
                        "@error": { "message": message, "type": ERROR_TYPE },
                        "@user": identity,
                        "supportKey": key,
                        "commandLine": command_line,
                        "settings": settings,
                    },
                }),
            })
            .collect();

        let _ = ureq::post(endpoint)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {API_KEY}"))
            .header(
                "User-Agent",
                concat!("curlgenerator/", env!("CARGO_PKG_VERSION")),
            )
            .send(Value::Array(events).to_string());
    }
}

/// Formats a point in time as an RFC 3339 UTC timestamp with nanosecond precision.
fn utc_timestamp(time: SystemTime) -> String {
    let since_epoch = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since_epoch.as_secs();
    let (hour, minute, second) = (seconds / 3600 % 24, seconds / 60 % 60, seconds % 60);

    // Howard Hinnant's `civil_from_days`, counting eras of 400 years from 1 March 0000.
    let shifted = i64::try_from(seconds / 86_400).unwrap_or_default() + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{:09}Z",
        since_epoch.subsec_nanos()
    )
}

/// Returns the long option names the invocation used.
///
/// # Examples
///
/// ```
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

#[cfg(test)]
mod tests {
    use super::*;

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

    type CapturedRequest = (String, Vec<(String, String)>, String);

    /// Accepts one HTTP request on a local port and hands back its request line, headers and body.
    fn capture_request() -> (String, std::sync::mpsc::Receiver<CapturedRequest>) {
        use std::io::{BufRead, Read, Write};

        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("a local port should be free");
        let endpoint = format!(
            "http://{}/api/v2/events",
            listener
                .local_addr()
                .expect("the listener should have an address")
        );
        let (sender, receiver) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            let (stream, _) = listener.accept().expect("a request should arrive");
            let mut reader = std::io::BufReader::new(stream);
            let mut request_line = String::new();
            reader.read_line(&mut request_line).expect("a request line");

            let mut headers = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).expect("a header line");
                match line.trim().split_once(':') {
                    Some((name, value)) => {
                        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()))
                    }
                    None => break,
                }
            }

            let length = headers
                .iter()
                .find(|(name, _)| name == "content-length")
                .and_then(|(_, value)| value.parse().ok())
                .unwrap_or(0);
            let mut body = vec![0; length];
            reader.read_exact(&mut body).expect("the body");

            let _ = reader.into_inner().write_all(
                b"HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
            let _ = sender.send((request_line, headers, String::from_utf8_lossy(&body).into()));
        });

        (endpoint, receiver)
    }

    #[test]
    fn submits_every_recorded_event_to_the_collector_in_one_request() {
        let args = parse(&["./openapi.json", "--bash"]);
        let mut telemetry = Telemetry::new(args.no_logging);
        telemetry.record_feature_usage(&args);
        telemetry.record_error("boom", &args);

        let (endpoint, requests) = capture_request();
        telemetry.flush_to(&endpoint);

        let (request_line, headers, body) = requests
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the events should be submitted");
        let header = |name: &str| {
            headers
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
        };

        assert!(
            request_line.starts_with("POST /api/v2/events "),
            "{request_line}"
        );
        assert_eq!(header("authorization"), Some(format!("Bearer {API_KEY}")));
        assert_eq!(header("content-type").as_deref(), Some("application/json"));
        assert!(telemetry.events().is_empty());

        let mut events: Vec<serde_json::Value> =
            serde_json::from_str(&body).expect("the body should be an array of events");
        for event in &mut events {
            let date = event
                .as_object_mut()
                .and_then(|event| event.remove("date"))
                .expect("every event should be dated");
            let date = date.as_str().expect("the date should be a string");

            assert_eq!(date.len(), "2026-01-01T00:00:00.000000000Z".len(), "{date}");
            assert!(date.ends_with('Z') && date[10..].starts_with('T'), "{date}");
        }

        let identity = anonymous_identity();
        let key = support_key();

        assert_eq!(
            events,
            vec![
                json!({
                    "type": "usage",
                    "source": "bash",
                    "data": { "@user": identity, "supportKey": key },
                }),
                json!({
                    "type": "usage",
                    "source": "content-type",
                    "data": { "@user": identity, "supportKey": key },
                }),
                json!({
                    "type": "error",
                    "source": "curlgenerator",
                    "tags": ["error"],
                    "data": {
                        "@error": {
                            "message": "boom",
                            "type": "curlgenerator::telemetry::GenerationError",
                        },
                        "@user": identity,
                        "supportKey": key,
                        "commandLine": redact_authorization_headers(&command_line()),
                        "settings": redacted_settings(&args),
                    },
                }),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn dates_submitted_events_in_utc() {
        let args = parse(&["./openapi.json"]);
        let mut telemetry = Telemetry::new(args.no_logging);
        telemetry.record_error("boom", &args);
        let utc = || {
            let output = std::process::Command::new("date")
                .args(["-u", "+%Y-%m-%dT%H:%M"])
                .output()
                .expect("the date command should run");

            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        let (endpoint, requests) = capture_request();
        let before = utc();
        telemetry.flush_to(&endpoint);
        let after = utc();

        let (_, _, body) = requests
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the event should be submitted");
        let events: serde_json::Value =
            serde_json::from_str(&body).expect("the body should be JSON");
        let date = events[0]["date"]
            .as_str()
            .expect("the event should be dated");

        assert!(
            date.starts_with(&before) || date.starts_with(&after),
            "{date} is not between {before} and {after}"
        );
    }
}
