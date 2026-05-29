//! String transformation helpers used for operation name and parameter generation.
//!
//! These mirror the behaviour of the original `StringExtensions` from the .NET
//! implementation so the generated output stays consistent.

/// Capitalizes the first character of the string, leaving the rest untouched.
pub fn capitalize_first_character(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Converts a kebab-case string to PascalCase, replacing `.` with `_`.
pub fn convert_kebab_case_to_pascal_case(value: &str) -> String {
    value
        .split('-')
        .map(|part| capitalize_first_character(part).replace('.', "_"))
        .collect()
}

/// Converts a kebab-case string to snake_case (lowercased).
pub fn convert_kebab_case_to_snake_case(value: &str) -> String {
    value.replace('-', "_").to_lowercase()
}

/// Converts a route (`/a/b/c`) to camel/Pascal-ish concatenation (`ABC`).
///
/// The first segment (before the leading slash) is kept as-is, every following
/// segment has its first character capitalized.
pub fn convert_route_to_camel_case(value: &str) -> String {
    let mut result = String::new();
    for (index, part) in value.split('/').enumerate() {
        if index == 0 {
            result.push_str(part);
        } else {
            result.push_str(&capitalize_first_character(part));
        }
    }
    result
}

/// Converts a space separated string to PascalCase.
pub fn convert_spaces_to_pascal_case(value: &str) -> String {
    value
        .split(' ')
        .map(capitalize_first_character)
        .collect()
}

/// Prepends `prefix` to `value` unless `value` already starts with it.
pub fn prefix(value: &str, prefix: &str) -> String {
    if value.starts_with(prefix) {
        value.to_string()
    } else {
        format!("{prefix}{value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_kebab_case_to_pascal_case_converts_correctly() {
        assert_eq!(convert_kebab_case_to_pascal_case("kebab-case-string"), "KebabCaseString");
        assert_eq!(
            convert_kebab_case_to_pascal_case("another-kebab-case.string"),
            "AnotherKebabCase_string"
        );
        assert_eq!(convert_kebab_case_to_pascal_case("single"), "Single");
    }

    #[test]
    fn convert_kebab_case_to_snake_case_converts_correctly() {
        assert_eq!(convert_kebab_case_to_snake_case("kebab-case-string"), "kebab_case_string");
        assert_eq!(
            convert_kebab_case_to_snake_case("another-kebab-case-string"),
            "another_kebab_case_string"
        );
        assert_eq!(convert_kebab_case_to_snake_case("single"), "single");
    }

    #[test]
    fn convert_route_to_camel_case_converts_correctly() {
        assert_eq!(convert_route_to_camel_case("/route/to/resource"), "RouteToResource");
        assert_eq!(
            convert_route_to_camel_case("/another/route/to/another/resource"),
            "AnotherRouteToAnotherResource"
        );
        assert_eq!(convert_route_to_camel_case("/single"), "Single");
    }

    #[test]
    fn capitalize_first_character_capitalizes_correctly() {
        assert_eq!(capitalize_first_character("string"), "String");
        assert_eq!(capitalize_first_character("anotherString"), "AnotherString");
        assert_eq!(capitalize_first_character("s"), "S");
        assert_eq!(capitalize_first_character(""), "");
    }

    #[test]
    fn convert_spaces_to_pascal_case_converts_correctly() {
        assert_eq!(convert_spaces_to_pascal_case("space separated string"), "SpaceSeparatedString");
        assert_eq!(
            convert_spaces_to_pascal_case("another space separated string"),
            "AnotherSpaceSeparatedString"
        );
        assert_eq!(convert_spaces_to_pascal_case("single"), "Single");
    }

    #[test]
    fn prefix_adds_prefix_correctly() {
        assert_eq!(prefix("string", "prefix"), "prefixstring");
        assert_eq!(prefix("prefixstring", "prefix"), "prefixstring");
    }
}
