//! Acquisition of an `Authorization` header from Azure Entra ID.
//!
//! Tokens come from the Azure CLI or the Azure Developer CLI, run the same way the developer
//! credentials of the Azure Identity SDK run them.

use std::process::Command;

use serde_json::Value;

/// The outcome of an Azure Entra ID token request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AzureAuth {
    /// No scope or tenant was configured, or an authorization header was already supplied.
    NotRequested,
    /// A bearer token was acquired.
    Acquired(String),
    /// The token could not be acquired.
    Failed(String),
}

/// Returns whether the invocation asks for an Azure Entra ID token.
///
/// An explicit `--authorization-header` always wins, matching the legacy CLI.
///
/// # Examples
///
/// ```
/// use curlgenerator::auth::wants_azure_token;
///
/// assert!(wants_azure_token(None, Some("api://scope/.default"), None));
/// assert!(!wants_azure_token(Some("Bearer token"), Some("api://scope/.default"), None));
/// assert!(!wants_azure_token(None, None, None));
/// ```
pub fn wants_azure_token(
    authorization_header: Option<&str>,
    azure_scope: Option<&str>,
    azure_tenant_id: Option<&str>,
) -> bool {
    let has_header = authorization_header.is_some_and(|value| !value.trim().is_empty());
    let has_scope = azure_scope.is_some_and(|value| !value.trim().is_empty());
    let has_tenant = azure_tenant_id.is_some_and(|value| !value.trim().is_empty());

    !has_header && (has_scope || has_tenant)
}

/// Requests an access token for `scope`, returning the `Authorization` header value.
///
/// The credential chain mirrors the legacy CLI: the Azure CLI for the configured tenant first,
/// then the Azure CLI for its default tenant, and finally the Azure Developer CLI. When none of
/// them succeeds, the failure of the first attempt is reported.
pub fn acquire_token(scope: &str, tenant_id: Option<&str>) -> AzureAuth {
    let tenant_id = tenant_id.filter(|tenant| !tenant.trim().is_empty());

    let failure = match azure_cli_token(scope, tenant_id) {
        Ok(token) => return AzureAuth::Acquired(format!("Bearer {token}")),
        Err(failure) => failure,
    };

    let default_tenant = match tenant_id {
        Some(_) => azure_cli_token(scope, None),
        None => Err(failure.clone()),
    };

    match default_tenant.or_else(|_| azure_developer_cli_token(scope)) {
        Ok(token) => AzureAuth::Acquired(format!("Bearer {token}")),
        Err(_) => AzureAuth::Failed(failure),
    }
}

fn azure_cli_token(scope: &str, tenant_id: Option<&str>) -> Result<String, String> {
    validate_scope(scope)?;

    let mut arguments = vec![
        "account",
        "get-access-token",
        "-o",
        "json",
        "--scope",
        scope,
    ];
    if let Some(tenant_id) = tenant_id {
        validate_tenant_id(tenant_id)?;
        arguments.extend(["--tenant", tenant_id]);
    }

    run_tool("az", "AzureCliCredential", &arguments, "accessToken")
}

fn azure_developer_cli_token(scope: &str) -> Result<String, String> {
    validate_scope(scope)?;

    run_tool(
        "azd",
        "AzureDeveloperCliCredential",
        &[
            "auth",
            "token",
            "-o",
            "json",
            "--no-prompt",
            "--scope",
            scope,
        ],
        "token",
    )
}

/// Runs a developer tool that prints a JSON token response, returning the token in `field`.
fn run_tool(
    program: &str,
    credential: &str,
    arguments: &[&str],
    field: &str,
) -> Result<String, String> {
    let not_found = || format!("{program} not found on PATH");

    let output = tool_command(program, arguments)
        .output()
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => not_found(),
            _ => format!("{credential} authentication failed. {error}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() == Some(127) || stderr.contains("' is not recognized") {
            return Err(not_found());
        }

        return Err(format!(
            "{credential} authentication failed. {}",
            stderr.trim()
        ));
    }

    serde_json::from_slice::<Value>(&output.stdout)
        .ok()
        .and_then(|response| response.get(field)?.as_str().map(str::to_string))
        .ok_or_else(|| format!("{credential} authentication failed. {program} returned no token"))
}

/// Builds the command that runs a developer tool from a fixed directory, so a tool planted in the
/// working directory is never picked up.
fn tool_command(program: &str, arguments: &[&str]) -> Command {
    if cfg!(windows) {
        // The tools are batch files on Windows, which only `cmd` runs. The arguments were
        // validated to contain nothing `cmd` interprets.
        let mut command = Command::new("cmd");
        command.arg("/C").arg(program).args(arguments);
        if let Some(system_root) = std::env::var_os("SYSTEMROOT") {
            command.current_dir(system_root);
        }

        command
    } else {
        let mut command = Command::new(program);
        command.args(arguments).current_dir("/");

        command
    }
}

fn validate_scope(scope: &str) -> Result<(), String> {
    let valid = !scope.is_empty()
        && scope.chars().all(|character| {
            character.is_alphanumeric() || matches!(character, '.' | '-' | '_' | ':' | '/')
        });

    valid
        .then_some(())
        .ok_or_else(|| format!("invalid scope {scope}"))
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), String> {
    let valid = !tenant_id.is_empty()
        && tenant_id
            .chars()
            .all(|character| character.is_alphanumeric() || matches!(character, '.' | '-'));

    valid.then_some(()).ok_or_else(|| {
        format!(
            "invalid tenant ID {tenant_id}. You can locate your tenant ID by following the \
             instructions listed here: https://learn.microsoft.com/partner-center/find-ids-and-domain-names"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_a_token_when_a_scope_or_tenant_is_configured() {
        assert!(wants_azure_token(None, Some("scope"), None));
        assert!(wants_azure_token(None, None, Some("tenant")));
        assert!(wants_azure_token(None, Some("scope"), Some("tenant")));
    }

    #[test]
    fn skips_the_token_when_an_authorization_header_was_supplied() {
        assert!(!wants_azure_token(
            Some("Bearer token"),
            Some("scope"),
            Some("tenant")
        ));
    }

    #[test]
    fn skips_the_token_without_any_azure_configuration() {
        assert!(!wants_azure_token(None, None, None));
        assert!(!wants_azure_token(None, Some("  "), Some("  ")));
    }
}
