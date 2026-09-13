//! Loading, decoding, and inspection of OpenAPI specifications.

mod loader;
mod normalize;
mod source;

pub use loader::load_document;
pub use normalize::{normalize, normalize_value};
pub use oasreader::{
    OpenApiContentFormat, OpenApiSource, OpenApiSpecificationVersion, OpenApiStats, ReadError,
    ReadResult, SourceClassificationError, classify_source, detect_specification_version,
    inspect_value,
};
pub use source::{authority_of, is_absolute_url, is_http};
