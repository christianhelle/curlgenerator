//! Redaction of sensitive values before anything is reported as telemetry.

/// The option whose value is redacted, matched case insensitively.
const OPTION: &str = "--authorization-header ";

const REPLACEMENT: &str = "--authorization-header [REDACTED]";

/// The shape of an authorization header value.
struct Pattern {
    /// The quote the value is wrapped in, if any.
    quote: Option<u8>,
    /// The number of space separated tokens the value consists of.
    tokens: usize,
}

/// The patterns are applied in order, from the most specific to the least specific, so a scheme
/// and its token are redacted together before the single token forms are considered.
///
/// They mirror the legacy regular expressions `--authorization-header "[^ ]+ [^ ]+"`,
/// `'[^ ]+ [^ ]+'`, `[^ ]+ [^ ]+`, `"[^ ]+"`, `'[^ ]+'` and `[^ ]+`, including their greedy runs.
const PATTERNS: [Pattern; 6] = [
    Pattern {
        quote: Some(b'"'),
        tokens: 2,
    },
    Pattern {
        quote: Some(b'\''),
        tokens: 2,
    },
    Pattern {
        quote: None,
        tokens: 2,
    },
    Pattern {
        quote: Some(b'"'),
        tokens: 1,
    },
    Pattern {
        quote: Some(b'\''),
        tokens: 1,
    },
    Pattern {
        quote: None,
        tokens: 1,
    },
];

/// Replaces any authorization header value in a command line with a placeholder.
///
/// # Examples
///
/// ```
/// use curlgenerator::privacy::redact_authorization_headers;
///
/// assert_eq!(
///     redact_authorization_headers("curlgenerator ./openapi.json --authorization-header Bearer secret"),
///     "curlgenerator ./openapi.json --authorization-header [REDACTED]"
/// );
/// ```
pub fn redact_authorization_headers(input: &str) -> String {
    PATTERNS.iter().fold(input.to_string(), |current, pattern| {
        pattern.replace_all(&current)
    })
}

impl Pattern {
    /// Replaces every non-overlapping match, scanning from left to right.
    fn replace_all(&self, input: &str) -> String {
        let bytes = input.as_bytes();
        let mut redacted = String::with_capacity(input.len());
        let mut copied = 0;
        let mut position = 0;

        while position + OPTION.len() <= bytes.len() {
            let value_end = if bytes[position..position + OPTION.len()]
                .eq_ignore_ascii_case(OPTION.as_bytes())
            {
                self.value_end(bytes, position + OPTION.len())
            } else {
                None
            };

            match value_end {
                Some(end) => {
                    redacted.push_str(&input[copied..position]);
                    redacted.push_str(REPLACEMENT);
                    copied = end;
                    position = end;
                }
                None => position += 1,
            }
        }

        redacted.push_str(&input[copied..]);
        redacted
    }

    /// Returns where a value of this shape starting at `start` ends.
    fn value_end(&self, bytes: &[u8], start: usize) -> Option<usize> {
        let mut position = start;

        if let Some(quote) = self.quote {
            if bytes.get(position) != Some(&quote) {
                return None;
            }
            position += 1;
        }

        for _ in 1..self.tokens {
            position = token_end(bytes, position)?;
            if bytes.get(position) != Some(&b' ') {
                return None;
            }
            position += 1;
        }

        let end = token_end(bytes, position)?;

        match self.quote {
            // The greedy run gives back characters until it ends on the closing quote, which must
            // leave at least one character of the token inside the quotes.
            Some(quote) => (position + 1..end)
                .rev()
                .find(|index| bytes[*index] == quote)
                .map(|index| index + 1),
            None => Some(end),
        }
    }
}

/// Returns the end of the non-empty run of non-space bytes starting at `start`.
fn token_end(bytes: &[u8], start: usize) -> Option<usize> {
    let end = bytes[start..]
        .iter()
        .position(|byte| *byte == b' ')
        .map_or(bytes.len(), |offset| start + offset);

    (end > start).then_some(end)
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

    #[test]
    fn applies_the_patterns_in_sequence_like_the_legacy_cli() {
        // Each pattern runs over the output of the previous one, so the two token form can consume
        // the argument that follows an already redacted value.
        assert_eq!(
            redact_authorization_headers(
                "curlgenerator ./openapi.json --authorization-header \"Bearer secret\" --bash"
            ),
            "curlgenerator ./openapi.json --authorization-header [REDACTED]"
        );
        assert_eq!(
            redact_authorization_headers(
                "curlgenerator ./openapi.json --authorization-header 'Basic dXNlcjpwYXNz' --output ./out"
            ),
            "curlgenerator ./openapi.json --authorization-header [REDACTED] ./out"
        );
    }

    #[test]
    fn leaves_an_option_without_a_value_untouched() {
        assert_eq!(
            redact_authorization_headers("curlgenerator --authorization-header"),
            "curlgenerator --authorization-header"
        );
    }
}
