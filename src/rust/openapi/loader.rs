//! Reading OpenAPI specifications from disk or over HTTP.

use oasreader::{DefaultLoader, HttpOptions, OpenApiReader, ReadError, ReadResult};

/// Loads a specification from a path or URL and merges the external references it contains.
///
/// Specifications split across multiple files or URLs are read into a single document. Remote
/// certificates are not verified, so specifications can be fetched from development servers with
/// self-signed certificates, matching the legacy CLI.
pub fn load_document(input: &str) -> Result<ReadResult, ReadError> {
    let loader = DefaultLoader::new(HttpOptions {
        accept_invalid_certificates: true,
        ..HttpOptions::default()
    });

    OpenApiReader::new().with_loader(loader).read(input)
}
