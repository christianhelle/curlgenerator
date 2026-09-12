//! Generation contracts checked against every specification in `test/OpenAPI`.

use std::{
    fs,
    path::{Path, PathBuf},
};

use curlgenerator::{
    GeneratorSettings,
    generator::generate,
    openapi::{load_document, normalize},
};

fn corpus() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/OpenAPI");
    let mut specifications = Vec::new();

    for version in fs::read_dir(&root).expect("the OpenAPI corpus should exist") {
        let version = version.expect("a readable corpus entry").path();
        if !version.is_dir() {
            continue;
        }

        for specification in fs::read_dir(&version).expect("a readable version directory") {
            let path = specification.expect("a readable specification").path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json" || extension == "yaml")
            {
                specifications.push(path);
            }
        }
    }

    specifications.sort();
    assert!(!specifications.is_empty(), "the corpus should not be empty");

    specifications
}

fn settings(path: &Path, bash: bool) -> GeneratorSettings {
    GeneratorSettings {
        open_api_path: path.to_string_lossy().to_string(),
        authorization_header: Some("Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()),
        generate_bash_scripts: bash,
        ..GeneratorSettings::new(path.to_string_lossy().to_string())
    }
}

#[test]
fn every_specification_generates_scripts() {
    for specification in corpus() {
        let result = generate(&settings(&specification, false))
            .unwrap_or_else(|error| panic!("{} failed: {error}", specification.display()));

        // A document that only declares webhooks has no operations to generate requests for.
        let document = normalize(
            &load_document(&specification.to_string_lossy()).expect("the document should load"),
        );
        let operations: usize = document
            .paths
            .iter()
            .map(|path_item| path_item.operations.len())
            .sum();

        assert_eq!(
            result.files.len(),
            operations,
            "{} generated {} files for {operations} operations",
            specification.display(),
            result.files.len()
        );
    }
}

#[test]
fn every_generated_script_carries_a_summary_comment() {
    for specification in corpus() {
        for bash in [false, true] {
            let result = generate(&settings(&specification, bash)).expect("generation should work");

            for file in &result.files {
                assert!(
                    file.content.chars().filter(|c| *c == '#').count() >= 2,
                    "{} in {} is missing its summary",
                    file.filename,
                    specification.display()
                );
            }
        }
    }
}

#[test]
fn every_generated_script_invokes_curl_with_the_authorization_header() {
    for specification in corpus() {
        for bash in [false, true] {
            let result = generate(&settings(&specification, bash)).expect("generation should work");

            for file in &result.files {
                assert!(
                    file.content.contains("curl -X "),
                    "{} does not invoke curl",
                    file.filename
                );
                assert!(
                    file.content.contains("Authorization: Bearer"),
                    "{} is missing the authorization header",
                    file.filename
                );
            }
        }
    }
}

#[test]
fn script_extensions_follow_the_selected_mode() {
    let specification =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/OpenAPI/v3.0/petstore.json");

    let powershell = generate(&settings(&specification, false)).expect("generation should work");
    let bash = generate(&settings(&specification, true)).expect("generation should work");

    assert!(
        powershell
            .files
            .iter()
            .all(|file| file.filename.ends_with(".ps1"))
    );
    assert!(bash.files.iter().all(|file| file.filename.ends_with(".sh")));
    assert_eq!(powershell.files.len(), bash.files.len());
}

#[test]
fn a_configured_base_url_is_applied_when_the_document_has_no_absolute_server() {
    let specification =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/OpenAPI/v3.0/petstore.json");

    let result = generate(&GeneratorSettings {
        base_url: Some("http://my-custom-base-url.com".to_string()),
        ..settings(&specification, false)
    })
    .expect("generation should work");

    assert!(
        result
            .files
            .iter()
            .all(|file| file.content.contains("http://my-custom-base-url.com"))
    );
}

#[test]
fn a_missing_specification_is_reported_as_an_error() {
    assert!(generate(&GeneratorSettings::new("./does-not-exist.json")).is_err());
}
