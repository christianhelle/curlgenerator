//! Derivation of script names from OpenAPI operations.

use std::collections::HashSet;

use crate::{
    normalized::Document,
    string_extensions::{
        capitalize_first_character, convert_kebab_case_to_pascal_case, convert_route_to_camel_case,
        convert_spaces_to_pascal_case, prefix,
    },
};

/// Returns the script name for an operation.
///
/// When the operation declares an identifier the name is derived from it and prefixed with the
/// capitalized, lower cased HTTP method. Otherwise the method is prepended to the path verbatim,
/// matching the legacy .NET `OperationNameGenerator`.
///
/// # Examples
///
/// ```
/// use curlgenerator::operation_name;
///
/// assert_eq!(operation_name("/my-path", "get", Some("my-operation")), "GetMyOperation");
/// assert_eq!(operation_name("/my-path", "GET", None), "GETMy-path");
/// ```
pub fn operation_name(path: &str, http_method: &str, operation_id: Option<&str>) -> String {
    match operation_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(operation_id) => {
            let name = convert_spaces_to_pascal_case(&convert_route_to_camel_case(
                &convert_kebab_case_to_pascal_case(&capitalize_first_character(operation_id)),
            ));

            prefix(
                &name,
                &capitalize_first_character(&http_method.to_lowercase()),
            )
        }
        None => {
            capitalize_first_character(http_method)
                + &convert_spaces_to_pascal_case(&convert_route_to_camel_case(path))
        }
    }
}

/// Returns `true` when two operations in the document resolve to the same script name.
///
/// # Examples
///
/// ```
/// use curlgenerator::{has_duplicate_operation_names, openapi::{OpenApiSpecificationVersion, normalize_value}};
/// use serde_json::json;
///
/// let document = normalize_value(
///     &json!({
///         "openapi": "3.0.0",
///         "paths": {
///             "/a": { "get": { "operationId": "list" } },
///             "/b": { "get": { "operationId": "list" } }
///         }
///     }),
///     OpenApiSpecificationVersion::OpenApi30,
/// );
///
/// assert!(has_duplicate_operation_names(&document));
/// ```
pub fn has_duplicate_operation_names(document: &Document) -> bool {
    let mut seen = HashSet::new();

    !document.paths.iter().all(|path_item| {
        path_item.operations.iter().all(|operation| {
            seen.insert(operation_name(
                &path_item.path,
                &operation.method,
                operation.operation_id.as_deref(),
            ))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::{OpenApiSpecificationVersion, normalize_value};
    use serde_json::{Value, json};

    fn document(value: &Value) -> Document {
        normalize_value(value, OpenApiSpecificationVersion::OpenApi30)
    }

    #[test]
    fn derives_the_name_from_the_operation_id() {
        assert_eq!(
            operation_name("/my-path", "get", Some("my-operation")),
            "GetMyOperation"
        );
    }

    #[test]
    fn falls_back_to_the_method_and_path_without_an_operation_id() {
        assert_eq!(operation_name("/my-path", "get", None), "GetMy-path");
        assert_eq!(
            operation_name("/estimates/price", "GET", None),
            "GETEstimatesPrice"
        );
        assert_eq!(operation_name("/my-path", "get", Some("   ")), "GetMy-path");
    }

    #[test]
    fn does_not_repeat_a_method_prefix_that_is_already_present() {
        assert_eq!(
            operation_name("/pets", "get", Some("getPetById")),
            "GetPetById"
        );
    }

    #[test]
    fn converts_spaces_and_routes_inside_operation_ids() {
        assert_eq!(
            operation_name("/pets", "post", Some("create pet/now")),
            "PostCreatePetNow"
        );
    }

    #[test]
    fn detects_duplicate_operation_names() {
        assert!(!has_duplicate_operation_names(&document(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/my-path": { "get": { "operationId": "my-operation" } },
                "/my-other-path": { "get": { "operationId": "my-other-operation" } }
            }
        }))));

        assert!(has_duplicate_operation_names(&document(&json!({
            "openapi": "3.0.0",
            "paths": {
                "/my-path": { "get": { "operationId": "my-operation" } },
                "/my-other-path": { "get": { "operationId": "my-operation" } }
            }
        }))));
    }
}
