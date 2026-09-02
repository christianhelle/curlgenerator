//! Rendering of PowerShell and Bash scripts from a normalized document.

mod bash;
mod powershell;
mod sample;
mod text;

pub use bash::render as render_bash;
pub use powershell::render as render_powershell;
pub use sample::{sample_json, sample_value};

/// The line ending generated scripts use, matching the host platform.
pub(crate) const NEWLINE: &str = if cfg!(windows) { "\r\n" } else { "\n" };
