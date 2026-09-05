//! Small text helpers shared by the script renderers.

/// Returns `true` when the value is empty or only contains whitespace.
///
/// This mirrors .NET's `string.IsNullOrWhiteSpace` for an already unwrapped value.
pub(crate) fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::is_blank;

    #[test]
    fn treats_whitespace_only_values_as_blank() {
        assert!(is_blank(""));
        assert!(is_blank("   \t\r\n"));
        assert!(!is_blank(" text "));
    }
}
