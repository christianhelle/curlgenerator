//! String helpers used when turning OpenAPI paths and operation ids into script names.
//!
//! Each helper mirrors the behaviour of the legacy .NET `StringExtensions` class so that both
//! implementations produce identical file names for the same specification.

/// Converts a kebab-case string to PascalCase, replacing `.` with `_`.
///
/// # Examples
///
/// ```
/// use curlgenerator::convert_kebab_case_to_pascal_case;
///
/// assert_eq!(convert_kebab_case_to_pascal_case("kebab-case-string"), "KebabCaseString");
/// ```
pub fn convert_kebab_case_to_pascal_case(value: &str) -> String {
    value
        .split('-')
        .map(|part| capitalize_first_character(part).replace('.', "_"))
        .collect()
}

/// Converts a kebab-case string to lowercase snake_case.
///
/// # Examples
///
/// ```
/// use curlgenerator::convert_kebab_case_to_snake_case;
///
/// assert_eq!(convert_kebab_case_to_snake_case("kebab-case"), "kebab_case");
/// ```
pub fn convert_kebab_case_to_snake_case(value: &str) -> String {
    value.replace('-', "_").to_lowercase()
}

/// Capitalizes every route segment after the leading separator and joins them together.
///
/// # Examples
///
/// ```
/// use curlgenerator::convert_route_to_camel_case;
///
/// assert_eq!(convert_route_to_camel_case("/route/to/resource"), "RouteToResource");
/// ```
pub fn convert_route_to_camel_case(value: &str) -> String {
    value
        .split('/')
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_string()
            } else {
                capitalize_first_character(part)
            }
        })
        .collect()
}

/// Uppercases the first character of the value and leaves the remainder untouched.
///
/// # Examples
///
/// ```
/// use curlgenerator::capitalize_first_character;
///
/// assert_eq!(capitalize_first_character("anotherString"), "AnotherString");
/// ```
pub fn capitalize_first_character(value: &str) -> String {
    let mut characters = value.chars();
    match characters.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
    }
}

/// Capitalizes every space separated word and joins them into a single PascalCase string.
///
/// # Examples
///
/// ```
/// use curlgenerator::convert_spaces_to_pascal_case;
///
/// assert_eq!(convert_spaces_to_pascal_case("space separated"), "SpaceSeparated");
/// ```
pub fn convert_spaces_to_pascal_case(value: &str) -> String {
    value.split(' ').map(capitalize_first_character).collect()
}

/// Prepends `prefix` unless the value already starts with it.
///
/// # Examples
///
/// ```
/// use curlgenerator::prefix;
///
/// assert_eq!(prefix("string", "prefix"), "prefixstring");
/// assert_eq!(prefix("prefixstring", "prefix"), "prefixstring");
/// ```
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
        assert_eq!(
            convert_kebab_case_to_pascal_case("kebab-case-string"),
            "KebabCaseString"
        );
        assert_eq!(
            convert_kebab_case_to_pascal_case("another-kebab-case.string"),
            "AnotherKebabCase_string"
        );
        assert_eq!(convert_kebab_case_to_pascal_case("single"), "Single");
    }

    #[test]
    fn convert_kebab_case_to_snake_case_converts_correctly() {
        assert_eq!(
            convert_kebab_case_to_snake_case("kebab-case-string"),
            "kebab_case_string"
        );
        assert_eq!(
            convert_kebab_case_to_snake_case("another-kebab-case-string"),
            "another_kebab_case_string"
        );
        assert_eq!(convert_kebab_case_to_snake_case("single"), "single");
    }

    #[test]
    fn convert_route_to_camel_case_converts_correctly() {
        assert_eq!(
            convert_route_to_camel_case("/route/to/resource"),
            "RouteToResource"
        );
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
        assert_eq!(
            convert_spaces_to_pascal_case("space separated string"),
            "SpaceSeparatedString"
        );
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
