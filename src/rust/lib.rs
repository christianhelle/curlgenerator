//! Library surface for the cURL Request Generator.
//!
//! This crate is the Rust port of the legacy `CurlGenerator` .NET application. It loads an
//! OpenAPI specification, normalizes it into a shared model, renders PowerShell or Bash scripts
//! that invoke `curl`, and exposes the command line plumbing that drives all of it.

#![deny(missing_docs)]

mod base_url;
pub mod generator;
mod model;
pub mod normalized;
pub mod openapi;

mod operation_name;

mod string_extensions;
mod support_information;

pub mod args;
pub mod auth;
pub mod help;
pub mod privacy;
pub mod run;
pub mod telemetry;
pub mod ui;
pub mod validation;

pub use base_url::base_url;
pub use model::{DEFAULT_CONTENT_TYPE, GeneratorResult, GeneratorSettings, ScriptFile};
pub use operation_name::{has_duplicate_operation_names, operation_name};
pub use string_extensions::{
    capitalize_first_character, convert_kebab_case_to_pascal_case,
    convert_kebab_case_to_snake_case, convert_route_to_camel_case, convert_spaces_to_pascal_case,
    prefix,
};
pub use support_information::{
    anonymous_identity, anonymous_identity_from_parts, support_key,
    support_key_from_anonymous_identity,
};
