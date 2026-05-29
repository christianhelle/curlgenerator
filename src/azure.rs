//! Acquires an Azure Entra ID access token via the Azure CLI (`az`).
//!
//! The original .NET tool used `ChainedTokenCredential` (Azure CLI first). This
//! re-implementation shells out to the Azure CLI, which covers the common case
//! of a developer already signed in with `az login`.

use std::process::Command;

use serde_json::Value;

/// Tries to acquire an access token for `scope` (and optional `tenant_id`).
///
/// Returns `Ok(None)` when the Azure CLI is unavailable or returns no token.
pub fn try_get_access_token(tenant_id: Option<&str>, scope: &str) -> Result<Option<String>, String> {
    let mut command = Command::new("az");
    command
        .arg("account")
        .arg("get-access-token")
        .arg("--scope")
        .arg(scope)
        .arg("--output")
        .arg("json");

    if let Some(tenant) = tenant_id.filter(|t| !t.trim().is_empty()) {
        command.arg("--tenant").arg(tenant);
    }

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => return Err(format!("Could not run the Azure CLI (az): {error}")),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Azure CLI failed to acquire a token: {}", stderr.trim()));
    }

    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Could not parse the Azure CLI response: {e}"))?;

    Ok(parsed
        .get("accessToken")
        .and_then(Value::as_str)
        .map(str::to_string))
}
