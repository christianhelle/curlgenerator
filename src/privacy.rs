//! Helpers for redacting sensitive information from text.

use regex::Regex;

/// Redacts any `--authorization-header ...` argument value from the input.
pub fn redact_authorization_headers(input: &str) -> String {
    const REPLACEMENT: &str = "--authorization-header [REDACTED]";
    let patterns = [
        r#"--authorization-header "[^ ]+ [^ ]+""#,
        r#"--authorization-header '[^ ]+ [^ ]+'"#,
        r#"--authorization-header [^ ]+ [^ ]+"#,
        r#"--authorization-header "[^ ]+""#,
        r#"--authorization-header '[^ ]+'"#,
        r#"--authorization-header [^ ]+"#,
    ];

    let mut result = input.to_string();
    for pattern in patterns {
        let regex = Regex::new(&format!("(?i){pattern}")).expect("valid regex");
        result = regex.replace_all(&result, REPLACEMENT).into_owned();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_authorization_header_variants() {
        let inputs = [
            "--authorization-header XxxxXxxxXxxx",
            "--authorization-header \"XxxxXxxxXxxx\"",
            "--authorization-header 'XxxxXxxxXxxx'",
            "--authorization-header Bearer XxxxXxxxXxxx",
            "--authorization-header Basic XxxxXxxxXxxx",
            "--authorization-header Token XxxxXxxxXxxx",
            "--authorization-header bearer XxxxXxxxXxxx",
            "--authorization-header 'Bearer XxxxXxxxXxxx'",
            "--authorization-header \"Bearer XxxxXxxxXxxx\"",
            "--authorization-header \"token XxxxXxxxXxxx\"",
        ];

        for input in inputs {
            assert_eq!(
                redact_authorization_headers(input),
                "--authorization-header [REDACTED]"
            );
        }
    }
}
