//! cURL Request Generator — generate cURL request scripts from OpenAPI specs.
//!
//! This library exposes the building blocks used by the `curlgenerator` CLI.

pub mod azure;
pub mod generator;
pub mod http;
pub mod model;
pub mod operation_name;
pub mod privacy;
pub mod strings;
pub mod support;
pub mod validation;

pub use generator::{generate, GeneratorResult, GeneratorSettings, ScriptFile};
pub use validation::{validate, OpenApiStats, ValidationResult};

#[cfg(test)]
mod tests;
