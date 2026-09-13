//! Loading, decoding, and inspection of OpenAPI specifications.

mod inspect;
mod loader;
mod normalize;
mod source;

pub use inspect::{OpenApiStats, inspect, inspect_value};
pub use loader::load_document;
pub use normalize::{normalize, normalize_value};
pub use oasreader::{
    OpenApiContentFormat, OpenApiSpecificationVersion, ReadError, ReadResult,
    detect_specification_version,
};
pub use source::{
    OpenApiSource, SourceClassificationError, authority_of, classify_source, is_absolute_url,
    is_http,
};
