//! cURL Request Generator — generate cURL request scripts from OpenAPI specs.
//!
//! This library exposes the building blocks used by the `curlgenerator` CLI.

pub mod generator;
pub mod http;
pub mod model;
pub mod operation_name;
pub mod strings;

pub use generator::{generate, GeneratorResult, GeneratorSettings, ScriptFile};
