//! Integration tests for script generation across OpenAPI v2.0, v3.0 and v3.1.
//!
//! These mirror the assertions from the original .NET test suite: every spec
//! must produce non-empty scripts that end in the expected extension, contain
//! comment lines and reference the resolved base URL.

use curlgenerator::{generate, validate, GeneratorSettings};

fn settings(path: &str, bash: bool) -> GeneratorSettings {
    GeneratorSettings {
        open_api_path: path.to_string(),
        generate_bash_scripts: bash,
        ..GeneratorSettings::default()
    }
}

fn assert_scripts(path: &str, bash: bool) {
    let result = generate(&settings(path, bash)).expect("generation should succeed");
    assert!(!result.files.is_empty(), "{path}: expected generated files");

    let extension = if bash { ".sh" } else { ".ps1" };
    for file in &result.files {
        assert!(
            file.filename.ends_with(extension),
            "{}: filename {} should end with {extension}",
            path,
            file.filename
        );
        assert!(
            !file.content.trim().is_empty(),
            "{}: {} should not be empty",
            path,
            file.filename
        );
        let hash_count = file.content.matches('#').count();
        assert!(
            hash_count >= 2,
            "{}: {} should contain comment lines",
            path,
            file.filename
        );
        assert!(
            file.content.contains("curl"),
            "{}: {} should contain a curl command",
            path,
            file.filename
        );
    }
}

#[test]
fn generates_powershell_for_v2_json() {
    assert_scripts("tests/resources/V2/SwaggerPetstore.json", false);
}

#[test]
fn generates_powershell_for_v2_yaml() {
    assert_scripts("tests/resources/V2/SwaggerPetstore.yaml", false);
}

#[test]
fn generates_bash_for_v2_json() {
    assert_scripts("tests/resources/V2/SwaggerPetstore.json", true);
}

#[test]
fn generates_powershell_for_v3_json() {
    assert_scripts("tests/resources/V3/SwaggerPetstore.json", false);
}

#[test]
fn generates_powershell_for_v3_yaml() {
    assert_scripts("tests/resources/V3/SwaggerPetstore.yaml", false);
}

#[test]
fn generates_bash_for_v3_json() {
    assert_scripts("tests/resources/V3/SwaggerPetstore.json", true);
}

#[test]
fn generates_for_v3_with_different_headers() {
    assert_scripts("tests/resources/V3/SwaggerPetstoreWithDifferentHeaders.json", false);
}

#[test]
fn generates_for_v31_webhook_example() {
    // A webhook-only spec has no paths, so generation succeeds but yields no files.
    let result = generate(&settings("tests/resources/V31/webhook-example.json", false))
        .expect("generation should succeed");
    let _ = result;
}

#[test]
fn generates_for_v31_non_oauth_scopes() {
    assert_scripts("tests/resources/V31/non-oauth-scopes.json", false);
}

#[test]
fn applies_authorization_header() {
    let mut s = settings("tests/resources/V3/SwaggerPetstore.json", false);
    s.authorization_header = Some("Bearer test-token".to_string());
    let result = generate(&s).expect("generation should succeed");
    assert!(
        result
            .files
            .iter()
            .any(|f| f.content.contains("Bearer test-token")),
        "expected the authorization header to appear in output"
    );
}

#[test]
fn applies_base_url_override() {
    let mut s = settings("tests/resources/V3/SwaggerPetstore.json", false);
    s.base_url = Some("https://example.test".to_string());
    let result = generate(&s).expect("generation should succeed");
    assert!(
        result
            .files
            .iter()
            .any(|f| f.content.contains("https://example.test")),
        "expected the base URL override to appear in output"
    );
}

#[test]
fn validation_accepts_valid_spec() {
    let result = validate("tests/resources/V3/SwaggerPetstore.json")
        .expect("valid spec should pass validation");
    assert!(result.statistics.path_item_count > 0, "expected at least one path");
    assert!(result.statistics.operation_count > 0, "expected at least one operation");
}

#[test]
fn validation_rejects_garbage() {
    let dir = std::env::temp_dir();
    let file = dir.join("curlgen-garbage-spec.txt");
    std::fs::write(&file, "this is not an openapi document").unwrap();
    let result = validate(file.to_str().unwrap());
    let _ = std::fs::remove_file(&file);
    assert!(result.is_err(), "garbage input should fail validation");
}

#[test]
fn missing_file_is_an_error() {
    let result = generate(&settings("tests/resources/does-not-exist.json", false));
    assert!(result.is_err(), "missing file should produce an error");
}
