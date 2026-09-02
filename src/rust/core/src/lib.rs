//! Core generation library for the cURL Request Generator.
//!
//! This crate is the Rust port of the legacy `CurlGenerator.Core` .NET library. It loads an
//! OpenAPI specification, normalizes it into a shared model, and renders PowerShell or Bash
//! scripts that invoke `curl`.

#![deny(missing_docs)]

pub mod openapi;

mod string_extensions;
mod support_information;

pub use string_extensions::{
    capitalize_first_character, convert_kebab_case_to_pascal_case, convert_kebab_case_to_snake_case,
    convert_route_to_camel_case, convert_spaces_to_pascal_case, prefix,
};
pub use support_information::{
    anonymous_identity, anonymous_identity_from_parts, support_key,
    support_key_from_anonymous_identity,
};
