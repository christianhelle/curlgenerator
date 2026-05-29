//! Generates an anonymous support key derived from the current user and machine.

use base64::Engine;
use sha2::{Digest, Sha256};

/// Returns the first 7 characters of the anonymous identity hash.
pub fn get_support_key() -> String {
    get_anonymous_identity().chars().take(7).collect()
}

/// Returns a SHA-256 (base64, lowercased) hash of `user@machine`.
pub fn get_anonymous_identity() -> String {
    let identity = format!("{}@{}", user_name(), machine_name());
    let hash = Sha256::digest(identity.as_bytes());
    base64::engine::general_purpose::STANDARD
        .encode(hash)
        .to_lowercase()
}

fn user_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "user".to_string())
}

fn machine_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_key_is_not_empty() {
        assert!(!get_support_key().is_empty());
        assert_eq!(get_support_key().len(), 7);
    }

    #[test]
    fn anonymous_identity_is_stable() {
        assert_eq!(get_anonymous_identity(), get_anonymous_identity());
    }
}
