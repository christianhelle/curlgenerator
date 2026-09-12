//! Command line arguments, mirroring the options of the legacy .NET CLI.

use std::{collections::HashMap, ffi::OsString, fmt, iter::Peekable, vec};

/// The default output directory.
pub const DEFAULT_OUTPUT: &str = "./";

/// The default `Content-Type` header applied to generated requests.
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// The usage line printed together with an argument error.
const USAGE: &str = "curlgenerator [URL or input file] [OPTIONS]";

/// An option the command line accepts.
struct Definition {
    long: &'static str,
    short: Option<char>,
    /// The name of the value the option takes, or `None` for a switch.
    value: Option<&'static str>,
}

impl Definition {
    fn display(&self) -> String {
        match self.value {
            Some(value) => format!("--{} <{value}>", self.long),
            None => format!("--{}", self.long),
        }
    }
}

const DEFINITIONS: [Definition; 11] = [
    Definition {
        long: "output",
        short: Some('o'),
        value: Some("OUTPUT"),
    },
    Definition {
        long: "bash",
        short: None,
        value: None,
    },
    Definition {
        long: "no-logging",
        short: None,
        value: None,
    },
    Definition {
        long: "skip-validation",
        short: None,
        value: None,
    },
    Definition {
        long: "authorization-header",
        short: None,
        value: Some("HEADER"),
    },
    Definition {
        long: "content-type",
        short: None,
        value: Some("CONTENT-TYPE"),
    },
    Definition {
        long: "base-url",
        short: None,
        value: Some("BASE-URL"),
    },
    Definition {
        long: "azure-scope",
        short: None,
        value: Some("SCOPE"),
    },
    Definition {
        long: "azure-tenant-id",
        short: None,
        value: Some("TENANT-ID"),
    },
    Definition {
        long: "help",
        short: Some('h'),
        value: None,
    },
    Definition {
        long: "version",
        short: Some('v'),
        value: None,
    },
];

/// Parsed command line arguments.
#[derive(Debug, Clone)]
pub struct Args {
    /// URL or file path to OpenAPI Specification file.
    pub open_api_path: Option<String>,
    /// Output directory.
    pub output: String,
    /// Generate Bash scripts.
    pub bash: bool,
    /// Don't log errors or collect telemetry.
    pub no_logging: bool,
    /// Skip validation of OpenAPI Specification file.
    pub skip_validation: bool,
    /// Authorization header to use for all requests.
    pub authorization_header: Option<String>,
    /// Default Content-Type header to use for all requests.
    pub content_type: String,
    /// Default Base URL to use for all requests.
    pub base_url: Option<String>,
    /// Azure Entra ID Scope to use for retrieving an access token.
    pub azure_scope: Option<String>,
    /// Azure Entra ID Tenant ID to use for retrieving an access token.
    pub azure_tenant_id: Option<String>,
    /// Prints help information.
    pub help: bool,
    /// Prints version information.
    pub version: bool,
}

/// An invocation the command line parser rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgsError(String);

impl fmt::Display for ArgsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for ArgsError {}

impl Args {
    /// Parses the arguments of the current process, exiting with code `2` when they are invalid.
    pub fn parse() -> Self {
        Self::try_parse_from(std::env::args_os()).unwrap_or_else(|error| {
            eprintln!("error: {error}\n\nUsage: {USAGE}\n\nFor more information, try '--help'.");
            std::process::exit(2);
        })
    }

    /// Parses `arguments`, whose first item is the program name.
    ///
    /// Options accept their value as the next argument, attached with `=`, or, for short options,
    /// directly after the letter. Everything after `--` is positional.
    ///
    /// # Examples
    ///
    /// ```
    /// use curlgenerator::args::Args;
    ///
    /// let args = Args::try_parse_from(["curlgenerator", "./openapi.json", "-o./out"]).unwrap();
    ///
    /// assert_eq!(args.output, "./out");
    /// assert!(Args::try_parse_from(["curlgenerator", "--bash=true"]).is_err());
    /// ```
    pub fn try_parse_from<I, T>(arguments: I) -> Result<Self, ArgsError>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString>,
    {
        let arguments = arguments
            .into_iter()
            .skip(1)
            .map(|argument| {
                argument.into().into_string().map_err(|_| {
                    ArgsError("invalid UTF-8 was detected in one or more arguments".to_string())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut parser = Parser::default();
        let mut remaining = arguments.into_iter().peekable();

        while let Some(argument) = remaining.next() {
            if argument == "--" {
                for positional in remaining.by_ref() {
                    parser.positional(positional)?;
                }
            } else if let Some(long) = argument.strip_prefix("--") {
                let (name, inline) = match long.split_once('=') {
                    Some((name, value)) => (name, Some(value.to_string())),
                    None => (long, None),
                };
                let definition = DEFINITIONS
                    .iter()
                    .find(|definition| definition.long == name)
                    .ok_or_else(|| unexpected(&argument))?;

                parser.option(definition, inline, &mut remaining)?;
            } else if let Some(shorts) = argument
                .strip_prefix('-')
                .filter(|shorts| !shorts.is_empty())
            {
                for (index, short) in shorts.char_indices() {
                    let definition = DEFINITIONS
                        .iter()
                        .find(|definition| definition.short == Some(short))
                        .ok_or_else(|| unexpected(&format!("-{short}")))?;

                    if definition.value.is_none() {
                        parser.option(definition, None, &mut remaining)?;
                        continue;
                    }

                    let attached = &shorts[index + short.len_utf8()..];
                    let inline = (!attached.is_empty())
                        .then(|| attached.strip_prefix('=').unwrap_or(attached).to_string());

                    parser.option(definition, inline, &mut remaining)?;
                    break;
                }
            } else {
                parser.positional(argument)?;
            }
        }

        let value = |long: &str| parser.values.get(long).cloned();
        let switch = |long: &str| parser.values.contains_key(long);

        Ok(Self {
            open_api_path: parser.open_api_path.clone(),
            output: value("output").unwrap_or_else(|| DEFAULT_OUTPUT.to_string()),
            bash: switch("bash"),
            no_logging: switch("no-logging"),
            skip_validation: switch("skip-validation"),
            authorization_header: value("authorization-header"),
            content_type: value("content-type").unwrap_or_else(|| DEFAULT_CONTENT_TYPE.to_string()),
            base_url: value("base-url"),
            azure_scope: value("azure-scope"),
            azure_tenant_id: value("azure-tenant-id"),
            help: switch("help"),
            version: switch("version"),
        })
    }

    /// Returns `true` when the invocation only asks for help.
    ///
    /// An invocation without any arguments prints help, matching the legacy CLI.
    pub fn wants_help(&self) -> bool {
        self.help || self.open_api_path.is_none()
    }
}

/// The options and the input collected so far.
#[derive(Default)]
struct Parser {
    open_api_path: Option<String>,
    /// The value of every option that was used, keyed by its long name. Switches store an empty
    /// value.
    values: HashMap<&'static str, String>,
}

impl Parser {
    fn positional(&mut self, argument: String) -> Result<(), ArgsError> {
        if self.open_api_path.is_some() {
            return Err(unexpected(&argument));
        }

        self.open_api_path = Some(argument);
        Ok(())
    }

    fn option(
        &mut self,
        definition: &Definition,
        inline: Option<String>,
        remaining: &mut Peekable<vec::IntoIter<String>>,
    ) -> Result<(), ArgsError> {
        if self.values.contains_key(definition.long) {
            return Err(ArgsError(format!(
                "the argument '{}' cannot be used multiple times",
                definition.display()
            )));
        }

        let value = match (definition.value, inline) {
            (None, None) => String::new(),
            (None, Some(value)) => {
                return Err(ArgsError(format!(
                    "unexpected value '{value}' for '{}' found; no more were expected",
                    definition.display()
                )));
            }
            (Some(_), Some(value)) => value,
            // A following option is not taken as the value, but a lone `-` is.
            (Some(_), None) => remaining
                .next_if(|next| next == "-" || !next.starts_with('-'))
                .ok_or_else(|| {
                    ArgsError(format!(
                        "a value is required for '{}' but none was supplied",
                        definition.display()
                    ))
                })?,
        };

        self.values.insert(definition.long, value);
        Ok(())
    }
}

fn unexpected(argument: &str) -> ArgsError {
    ArgsError(format!("unexpected argument '{argument}' found"))
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

    #[test]
    fn accepts_attached_option_values() {
        assert_eq!(parse(&["./openapi.json", "--output=./a"]).output, "./a");
        assert_eq!(parse(&["./openapi.json", "-o./b"]).output, "./b");
        assert_eq!(parse(&["./openapi.json", "-o=./c"]).output, "./c");
        assert_eq!(
            parse(&["./openapi.json", "--content-type=text/plain"]).content_type,
            "text/plain"
        );
    }

    #[test]
    fn treats_everything_after_a_double_dash_as_the_input() {
        assert_eq!(
            parse(&["--", "-openapi.json"]).open_api_path.as_deref(),
            Some("-openapi.json")
        );
    }

    #[test]
    fn combines_short_flags() {
        let args = parse(&["-vh"]);

        assert!(args.version);
        assert!(args.help);
    }

    #[test]
    fn rejects_invalid_invocations() {
        for arguments in [
            &["--unknown"][..],
            &["./openapi.json", "extra"],
            &["./openapi.json", "--bash", "--bash"],
            &["./openapi.json", "-o", "a", "-o", "b"],
            &["./openapi.json", "--bash=true"],
            &["./openapi.json", "--output"],
            &["./openapi.json", "--output", "--bash"],
        ] {
            let mut all = vec!["curlgenerator"];
            all.extend_from_slice(arguments);

            assert!(
                Args::try_parse_from(all).is_err(),
                "{arguments:?} should be rejected"
            );
        }
    }
}
