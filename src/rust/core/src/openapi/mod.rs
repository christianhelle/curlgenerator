//! Loading, decoding, and inspection of OpenAPI specifications.

mod loader;
mod normalize;
mod source;

pub use loader::{
    OpenApiContentFormat, OpenApiLoadError, OpenApiSpecificationVersion, RawOpenApiDocument,
    decode_document, detect_specification_version, load_document,
};
pub use normalize::{normalize, normalize_value};
pub use source::{
    OpenApiSource, SourceClassificationError, authority_of, classify_source, is_http,
};
