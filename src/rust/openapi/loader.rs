//! Reading OpenAPI specifications from disk or over HTTP.

use oasreader::{DefaultLoader, HttpOptions, OpenApiReader, ReadError, ReadResult};

/// Loads a specification from a path or URL and merges the external references it contains.
///
/// Specifications split across multiple files or URLs are read into a single document. Remote
/// certificates are verified unless `accept_invalid_certificates` is set, which allows fetching
/// specifications from development servers with self-signed certificates.
pub fn load_document(
    input: &str,
    accept_invalid_certificates: bool,
) -> Result<ReadResult, ReadError> {
    let loader = DefaultLoader::new(HttpOptions {
        accept_invalid_certificates,
        ..HttpOptions::default()
    });

    OpenApiReader::new().with_loader(loader).read(input)
}
