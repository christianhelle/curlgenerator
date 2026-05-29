//! Generates a unique-ish operation name used as the script file name.

use crate::model::Document;
use crate::strings::{
    capitalize_first_character, convert_kebab_case_to_pascal_case, convert_route_to_camel_case,
    convert_spaces_to_pascal_case, prefix,
};

/// Generates the operation name from the `operationId` (preferred) or, when it
/// is missing, from the HTTP method and path.
pub fn get_operation_name(path: &str, http_method: &str, operation_id: Option<&str>) -> String {
    match operation_id {
        Some(id) if !id.trim().is_empty() => {
            let transformed = convert_spaces_to_pascal_case(&convert_route_to_camel_case(
                &convert_kebab_case_to_pascal_case(&capitalize_first_character(id)),
            ));
            let verb_prefix = capitalize_first_character(&http_method.to_lowercase());
            prefix(&transformed, &verb_prefix)
        }
        _ => {
            capitalize_first_character(http_method)
                + &convert_spaces_to_pascal_case(&convert_route_to_camel_case(path))
        }
    }
}

/// Returns `true` if any two operations in the document resolve to the same name.
pub fn has_duplicate_operation_ids(document: &Document) -> bool {
    let mut names = Vec::new();
    for path_item in &document.paths {
        for operation in &path_item.operations {
            names.push(get_operation_name(
                &path_item.path,
                &operation.method,
                operation.operation_id.as_deref(),
            ));
        }
    }

    let total = names.len();
    names.sort();
    names.dedup();
    names.len() != total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Operation, PathItem};
    use serde_json::Value;

    fn document(paths: Vec<PathItem>) -> Document {
        Document {
            servers: Vec::new(),
            paths,
            is_v2: false,
            root: Value::Null,
        }
    }

    fn operation(method: &str, operation_id: Option<&str>) -> Operation {
        Operation {
            method: method.to_string(),
            operation_id: operation_id.map(str::to_string),
            summary: None,
            description: None,
            parameters: Vec::new(),
            request_body: None,
        }
    }

    #[test]
    fn returns_expected_name_for_valid_input() {
        let result = get_operation_name("/my-path", "get", Some("my-operation"));
        assert_eq!(result, "GetMyOperation");
    }

    #[test]
    fn returns_fallback_name_when_no_operation_id() {
        let result = get_operation_name("/my-path", "get", None);
        assert_eq!(result, "GetMy-path");
    }

    #[test]
    fn check_for_duplicate_operation_ids_no_duplicates() {
        let doc = document(vec![
            PathItem {
                path: "/my-path".to_string(),
                operations: vec![operation("get", Some("my-operation"))],
            },
            PathItem {
                path: "/my-other-path".to_string(),
                operations: vec![operation("get", Some("my-other-operation"))],
            },
        ]);
        assert!(!has_duplicate_operation_ids(&doc));
    }

    #[test]
    fn check_for_duplicate_operation_ids_with_duplicates() {
        let doc = document(vec![
            PathItem {
                path: "/my-path".to_string(),
                operations: vec![operation("get", Some("my-operation"))],
            },
            PathItem {
                path: "/my-other-path".to_string(),
                operations: vec![operation("get", Some("my-operation"))],
            },
        ]);
        assert!(has_duplicate_operation_ids(&doc));
    }
}
