//! The end to end generation pipeline.

use crate::{
    base_url::base_url,
    generator::{bash, powershell},
    model::{GeneratorResult, GeneratorSettings, ScriptFile},
    normalized::Document,
    openapi::{OpenApiLoadError, load_document, normalize},
    operation_name::operation_name,
    string_extensions::capitalize_first_character,
};

/// Loads the configured specification and renders a script per operation.
pub fn generate(settings: &GeneratorSettings) -> Result<GeneratorResult, OpenApiLoadError> {
    let raw = load_document(&settings.open_api_path)?;

    Ok(generate_from_document(settings, &normalize(&raw)))
}

/// Renders a script per operation of an already normalized document.
///
/// # Examples
///
/// ```
/// use curlgenerator::{
///     GeneratorSettings,
///     generator::generate_from_document,
///     openapi::{OpenApiSpecificationVersion, normalize_value},
/// };
/// use serde_json::json;
///
/// let document = normalize_value(
///     &json!({
///         "openapi": "3.0.0",
///         "paths": { "/pets": { "get": { "operationId": "listPets" } } }
///     }),
///     OpenApiSpecificationVersion::OpenApi30,
/// );
///
/// let result = generate_from_document(&GeneratorSettings::new("./openapi.json"), &document);
///
/// assert_eq!(result.files[0].filename, "GetListPets.ps1");
/// ```
pub fn generate_from_document(
    settings: &GeneratorSettings,
    document: &Document,
) -> GeneratorResult {
    let base_url = base_url(
        document,
        settings.base_url.as_deref(),
        &settings.open_api_path,
    );
    let extension = settings.script_extension();

    let files = document
        .paths
        .iter()
        .flat_map(|path_item| {
            let base_url = &base_url;

            path_item.operations.iter().map(move |operation| {
                let name = operation_name(
                    &path_item.path,
                    &operation.method,
                    operation.operation_id.as_deref(),
                );
                let content = if settings.generate_bash_scripts {
                    bash::render(settings, base_url, &path_item.path, operation)
                } else {
                    powershell::render(settings, base_url, &path_item.path, operation)
                };

                ScriptFile::new(
                    format!("{}.{extension}", capitalize_first_character(&name)),
                    content,
                )
            })
        })
        .collect();

    GeneratorResult { files }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::{OpenApiSpecificationVersion, normalize_value};
    use serde_json::{Value, json};

    fn petstore() -> Value {
        json!({
            "openapi": "3.0.0",
            "servers": [{ "url": "/api/v3" }],
            "paths": {
                "/pet/{petId}": {
                    "get": { "operationId": "getPetById" },
                    "delete": { "operationId": "deletePet" }
                },
                "/pet/findByStatus": { "get": { "operationId": "findPetsByStatus" } }
            }
        })
    }

    fn generate_petstore(settings: &GeneratorSettings) -> GeneratorResult {
        let document = normalize_value(&petstore(), OpenApiSpecificationVersion::OpenApi30);

        generate_from_document(settings, &document)
    }

    #[test]
    fn generates_one_powershell_script_per_operation_in_document_order() {
        let result = generate_petstore(&GeneratorSettings::new("./openapi.json"));

        assert_eq!(
            result
                .files
                .iter()
                .map(|file| file.filename.as_str())
                .collect::<Vec<_>>(),
            vec!["GetPetById.ps1", "DeletePet.ps1", "GetFindPetsByStatus.ps1"]
        );
    }

    #[test]
    fn generates_bash_scripts_when_requested() {
        let result = generate_petstore(&GeneratorSettings {
            generate_bash_scripts: true,
            ..GeneratorSettings::new("./openapi.json")
        });

        assert!(
            result
                .files
                .iter()
                .all(|file| file.filename.ends_with(".sh"))
        );
        assert!(result.files[0].content.starts_with("#"));
    }

    #[test]
    fn applies_the_resolved_base_url_to_every_request() {
        let result = generate_petstore(&GeneratorSettings::new("./openapi.json"));

        assert!(
            result
                .files
                .iter()
                .all(|file| file.content.contains("/api/v3/pet"))
        );
    }

    #[test]
    fn prefers_the_configured_base_url() {
        let result = generate_petstore(&GeneratorSettings {
            base_url: Some("http://my-custom-base-url.com".to_string()),
            ..GeneratorSettings::new("./openapi.json")
        });

        assert!(result.files.iter().all(|file| {
            file.content
                .contains("http://my-custom-base-url.com/api/v3")
        }));
    }

    #[test]
    fn reports_a_load_error_for_a_missing_specification() {
        let error = generate(&GeneratorSettings::new("./does-not-exist.json")).unwrap_err();

        assert!(matches!(error, OpenApiLoadError::FileRead { .. }));
    }
}
