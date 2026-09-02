//! Loading, decoding, and inspection of OpenAPI specifications.

mod source;

pub use source::{
    OpenApiSource, SourceClassificationError, authority_of, classify_source, is_http,
};
