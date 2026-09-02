//! Acquisition of an `Authorization` header from Azure Entra ID.

use azure_core::credentials::TokenCredential;
use azure_identity::{AzureCliCredential, DeveloperToolsCredential};

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
/// use curlgenerator_cli::auth::wants_azure_token;
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
/// The credential chain mirrors the legacy CLI: the Azure CLI first, then the credentials the
/// local developer tooling provides.
pub async fn acquire_token(scope: &str, tenant_id: Option<&str>) -> AzureAuth {
    let scopes = [scope];

    let failure = match azure_cli_credential(tenant_id) {
        Ok(credential) => match credential.get_token(&scopes, None).await {
            Ok(token) => return AzureAuth::Acquired(format!("Bearer {}", token.token.secret())),
            Err(error) => error.to_string(),
        },
        Err(error) => error.to_string(),
    };

    match developer_tools_token(&scopes).await {
        Some(token) => AzureAuth::Acquired(token),
        None => AzureAuth::Failed(failure),
    }
}

fn azure_cli_credential(
    tenant_id: Option<&str>,
) -> azure_core::Result<std::sync::Arc<AzureCliCredential>> {
    let options = tenant_id
        .filter(|tenant| !tenant.trim().is_empty())
        .map(|tenant| azure_identity::AzureCliCredentialOptions {
            tenant_id: Some(tenant.to_string()),
            ..Default::default()
        });

    AzureCliCredential::new(options)
}

async fn developer_tools_token(scopes: &[&str]) -> Option<String> {
    let credential = DeveloperToolsCredential::new(None).ok()?;
    let token = credential.get_token(scopes, None).await.ok()?;

    Some(format!("Bearer {}", token.token.secret()))
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
