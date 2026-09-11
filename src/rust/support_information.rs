//! Helpers for producing anonymous support identifiers.
//!
//! The identity is a SHA-256 hash of `user@machine`, Base64 encoded and lowercased, exactly as
//! the legacy .NET `SupportInformation` class produces it.

use std::{env, ffi::OsString};

/// The SHA-256 round constants.
const ROUND_CONSTANTS: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The SHA-256 initial hash value.
const INITIAL_STATE: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// The standard Base64 alphabet.
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Returns a stable anonymous identity derived from the current user and machine.
pub fn anonymous_identity() -> String {
    let user_name = current_user_name();
    let machine_name = current_machine_name();

    anonymous_identity_from_parts(&user_name, machine_name.as_deref())
}

/// Returns the anonymous support identity for explicit user and machine parts.
///
/// Empty or missing machine names fall back to `"localhost"`, matching the legacy .NET
/// implementation.
///
/// # Examples
///
/// ```
/// use curlgenerator::anonymous_identity_from_parts;
///
/// assert_eq!(
///     anonymous_identity_from_parts("alice", Some("build-agent")),
///     "prihjx2hffzjfsy4vly5/8ynzks7bznfs3wk4b+e+xm="
/// );
/// ```
pub fn anonymous_identity_from_parts(user_name: &str, machine_name: Option<&str>) -> String {
    let machine_name = machine_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("localhost");
    let value = format!("{user_name}@{machine_name}");

    base64(&sha256(value.as_bytes())).to_ascii_lowercase()
}

/// Returns the SHA-256 digest of `bytes`.
fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&(bytes.len() as u64 * 8).to_be_bytes());

    let mut state = INITIAL_STATE;
    for block in message.as_chunks::<64>().0 {
        let mut words = [0u32; 64];
        for (word, chunk) in words.iter_mut().zip(block.as_chunks::<4>().0) {
            *word = u32::from_be_bytes(*chunk);
        }
        for index in 16..64 {
            let distant = words[index - 15];
            let recent = words[index - 2];
            words[index] = words[index - 16]
                .wrapping_add(distant.rotate_right(7) ^ distant.rotate_right(18) ^ (distant >> 3))
                .wrapping_add(words[index - 7])
                .wrapping_add(recent.rotate_right(17) ^ recent.rotate_right(19) ^ (recent >> 10));
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for (word, constant) in words.into_iter().zip(ROUND_CONSTANTS) {
            let first = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(constant)
                .wrapping_add(word);
            let second = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(first);
            d = c;
            c = b;
            b = a;
            a = first.wrapping_add(second);
        }

        for (value, compressed) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *value = value.wrapping_add(compressed);
        }
    }

    let mut digest = [0u8; 32];
    for (chunk, value) in digest.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        *chunk = value.to_be_bytes();
    }

    digest
}

/// Encodes `bytes` as padded standard Base64.
fn base64(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let group = chunk.iter().enumerate().fold(0u32, |group, (index, byte)| {
            group | (u32::from(*byte) << (16 - 8 * index))
        });

        for position in 0..4 {
            if position <= chunk.len() {
                let index = (group >> (18 - 6 * position)) & 0x3f;
                encoded.push(char::from(BASE64_ALPHABET[index as usize]));
            } else {
                encoded.push('=');
            }
        }
    }

    encoded
}

/// Returns the short support key for the current anonymous identity.
pub fn support_key() -> String {
    support_key_from_anonymous_identity(&anonymous_identity())
}

/// Returns the short support key associated with an anonymous identity.
///
/// The support key is the first seven characters of the full anonymous identity.
///
/// # Examples
///
/// ```
/// use curlgenerator::support_key_from_anonymous_identity;
///
/// assert_eq!(
///     support_key_from_anonymous_identity("prihjx2hffzjfsy4vly5/8ynzks7bznfs3wk4b+e+xm="),
///     "prihjx2"
/// );
/// ```
pub fn support_key_from_anonymous_identity(anonymous_identity: &str) -> String {
    anonymous_identity.chars().take(7).collect()
}

fn current_user_name() -> String {
    env_value(&["USERNAME", "USER", "LOGNAME"]).unwrap_or_default()
}

fn current_machine_name() -> Option<String> {
    hostname::get()
        .ok()
        .and_then(normalize_os_string)
        .or_else(|| env_value(&["COMPUTERNAME", "HOSTNAME"]))
}

fn env_value(keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| env::var_os(key).and_then(normalize_os_string))
}

fn normalize_os_string(value: OsString) -> Option<String> {
    let value = value.to_string_lossy();
    let value = value.trim();

    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{anonymous_identity_from_parts, support_key, support_key_from_anonymous_identity};

    #[test]
    fn anonymous_identity_matches_dotnet_sha256_base64_lowercase() {
        let identity = anonymous_identity_from_parts("alice", Some("build-agent"));

        assert_eq!(identity, "prihjx2hffzjfsy4vly5/8ynzks7bznfs3wk4b+e+xm=");
        assert_eq!(identity.len(), 44);
    }

    #[test]
    fn anonymous_identity_falls_back_to_localhost_when_machine_name_is_missing() {
        let expected = "o22kzws2q0n0j9qajmfa/dm8puf5ilfqxfxdv4c49so=";

        assert_eq!(anonymous_identity_from_parts("octocat", None), expected);
        assert_eq!(
            anonymous_identity_from_parts("octocat", Some("   ")),
            expected
        );
    }

    #[test]
    fn support_key_uses_first_seven_characters_of_anonymous_identity() {
        assert_eq!(
            support_key_from_anonymous_identity("prihjx2hffzjfsy4vly5/8ynzks7bznfs3wk4b+e+xm="),
            "prihjx2"
        );
    }

    #[test]
    fn support_key_is_not_empty() {
        assert_eq!(support_key().len(), 7);
    }

    #[test]
    fn anonymous_identity_hashes_values_across_sha256_block_boundaries() {
        // `user@build-agent` is 55, 56, 64 and 120 bytes long, straddling the padding boundaries.
        for (user_length, expected) in [
            (43, "46xfwrqto5ihrm8r74xiclym8khkzilhwihfwtj2/ic="),
            (44, "kxz49criov89syiupji7chh0pnnj2ztoiifvea5n000="),
            (52, "w+9rpscdbhkscttxiuienlfiw32jjtpzqrlvnnm1g04="),
            (108, "hdnygea3yotby9capap7hmra6ax0500jqubgeou1uti="),
        ] {
            assert_eq!(
                anonymous_identity_from_parts(&"a".repeat(user_length), Some("build-agent")),
                expected,
                "failed for a {user_length} character user name"
            );
        }
    }

    #[test]
    fn anonymous_identity_hashes_the_utf8_bytes_of_the_user_name() {
        assert_eq!(
            anonymous_identity_from_parts("Søren", Some("build-agent")),
            "camwml74lg9zl3spx8bhiyterdidgqbkhzqgsiirflc="
        );
    }
}
