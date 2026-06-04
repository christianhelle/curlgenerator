pub fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    s[..1].to_uppercase() + &s[1..]
}

pub fn convert_kebab_to_pascal(s: &str) -> String {
    s.split('-')
        .map(|part| capitalize_first(part).replace('.', "_"))
        .collect()
}

pub fn convert_kebab_to_snake(s: &str) -> String {
    s.replace('-', "_").to_lowercase()
}

pub fn convert_route_to_camel(s: &str) -> String {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() <= 1 {
        return s.to_string();
    }
    let mut result = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            result.push_str(part);
        } else {
            result.push_str(&capitalize_first(part));
        }
    }
    result
}

pub fn convert_spaces_to_pascal(s: &str) -> String {
    s.split(' ')
        .map(|part| capitalize_first(part))
        .collect()
}

pub fn prefix(s: &str, p: &str) -> String {
    if s.starts_with(p) {
        s.to_string()
    } else {
        format!("{}{}", p, s)
    }
}

#[derive(Debug, Clone)]
pub struct OperationNameGenerator;

impl OperationNameGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn get_operation_name(
        &self,
        path: &str,
        http_method: &str,
        operation: &openapiv3::Operation,
    ) -> String {
        if let Some(ref operation_id) = operation.operation_id {
            if !operation_id.is_empty() {
                let name = capitalize_first(operation_id);
                let name = convert_kebab_to_pascal(&name);
                let name = convert_route_to_camel(&name);
                let name = convert_spaces_to_pascal(&name);
                let method_prefix = capitalize_first(http_method);
                return prefix(&name, &method_prefix);
            }
        }

        let method_prefix = capitalize_first(http_method);
        let path_part = convert_route_to_camel(&convert_spaces_to_pascal(path));
        format!("{}{}", method_prefix, path_part)
    }
}

pub fn append_summary(
    verb: &str,
    path: &str,
    operation: &openapiv3::Operation,
    code: &mut String,
    is_powershell: bool,
) {
    if is_powershell {
        code.push_str("<#\n");
        code.push_str(&format!("  Request: {} {}\n", verb.to_uppercase(), path));
        if let Some(ref summary) = operation.summary {
            code.push_str(&format!("  Summary: {}\n", summary));
        }
        if let Some(ref description) = operation.description {
            code.push_str(&format!("  Description: {}\n", description));
        }
        code.push_str("#>\n");
    } else {
        code.push_str("#\n");
        code.push_str(&format!("# Request: {} {}\n", verb.to_uppercase(), path));
        if let Some(ref summary) = operation.summary {
            code.push_str(&format!("# Summary: {}\n", summary));
        }
        if let Some(ref description) = operation.description {
            for line in description.lines() {
                code.push_str(&format!("# {}\n", line.trim()));
            }
        }
        code.push_str("#\n");
    }
}
