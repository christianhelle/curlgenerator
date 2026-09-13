//! Loading, decoding, and inspection of OpenAPI specifications.

mod loader;
mod normalize;
mod source;

pub use loader::load_document;
pub use normalize::{normalize, normalize_value};
pub use oasreader::{
    OpenApiContentFormat, OpenApiSpecificationVersion, OpenApiStats, ReadError, ReadResult,
    detect_specification_version, inspect_value,
};
pub use source::{
    OpenApiSource, SourceClassificationError, authority_of, classify_source, is_absolute_url,
    is_http,
};
