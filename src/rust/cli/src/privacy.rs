//! Redaction of sensitive values before anything is reported as telemetry.

use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

const REPLACEMENT: &str = "--authorization-header [REDACTED]";

/// The patterns are applied in order, from the most specific to the least specific, so a scheme
/// and its token are redacted together before the single token forms are considered.
const PATTERNS: [&str; 6] = [
    r#"--authorization-header "[^ ]+ [^ ]+""#,
    r"--authorization-header '[^ ]+ [^ ]+'",
    r"--authorization-header [^ ]+ [^ ]+",
    r#"--authorization-header "[^ ]+""#,
    r"--authorization-header '[^ ]+'",
    r"--authorization-header [^ ]+",
];

static REDACTIONS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    PATTERNS
        .iter()
        .map(|pattern| {
            RegexBuilder::new(pattern)
                .case_insensitive(true)
                .build()
                .expect("the redaction patterns are valid")
        })
        .collect()
});

/// Replaces any authorization header value in a command line with a placeholder.
///
/// # Examples
///
/// ```
/// use curlgenerator_cli::privacy::redact_authorization_headers;
///
/// assert_eq!(
///     redact_authorization_headers("curlgenerator ./openapi.json --authorization-header Bearer secret"),
///     "curlgenerator ./openapi.json --authorization-header [REDACTED]"
/// );
/// ```
pub fn redact_authorization_headers(input: &str) -> String {
    REDACTIONS
        .iter()
        .fold(input.to_string(), |current, pattern| {
            pattern.replace_all(&current, REPLACEMENT).into_owned()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_every_documented_authorization_header_form() {
        for input in [
            "--authorization-header XxxxXxxxXxxx",
            "--authorization-header \"XxxxXxxxXxxx\"",
            "--authorization-header 'XxxxXxxxXxxx'",
            "--authorization-header Bearer XxxxXxxxXxxx",
            "--authorization-header Basic XxxxXxxxXxxx",
            "--authorization-header Token XxxxXxxxXxxx",
            "--authorization-header bearer XxxxXxxxXxxx",
            "--authorization-header basic XxxxXxxxXxxx",
            "--authorization-header token XxxxXxxxXxxx",
            "--authorization-header 'Bearer XxxxXxxxXxxx'",
            "--authorization-header 'Basic XxxxXxxxXxxx'",
            "--authorization-header 'Token XxxxXxxxXxxx'",
            "--authorization-header 'bearer XxxxXxxxXxxx'",
            "--authorization-header 'basic XxxxXxxxXxxx'",
            "--authorization-header 'token XxxxXxxxXxxx'",
            "--authorization-header \"Bearer XxxxXxxxXxxx\"",
            "--authorization-header \"Basic XxxxXxxxXxxx\"",
            "--authorization-header \"Token XxxxXxxxXxxx\"",
            "--authorization-header \"bearer XxxxXxxxXxxx\"",
            "--authorization-header \"basic XxxxXxxxXxxx\"",
            "--authorization-header \"token XxxxXxxxXxxx\"",
        ] {
            assert_eq!(
                redact_authorization_headers(input),
                REPLACEMENT,
                "failed to redact {input}"
            );
        }
    }

    #[test]
    fn matches_the_option_case_insensitively() {
        assert_eq!(
            redact_authorization_headers("--AUTHORIZATION-HEADER Bearer secret"),
            REPLACEMENT
        );
    }

    #[test]
    fn leaves_other_arguments_untouched() {
        assert_eq!(
            redact_authorization_headers("curlgenerator ./openapi.json --bash"),
            "curlgenerator ./openapi.json --bash"
        );
    }

    #[test]
    fn is_idempotent() {
        let once = redact_authorization_headers("--authorization-header Bearer secret");

        assert_eq!(redact_authorization_headers(&once), once);
    }
}
