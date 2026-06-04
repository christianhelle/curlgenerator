use std::fmt;

#[derive(Clone, Debug)]
pub struct ScriptFile {
    pub filename: String,
    pub content: String,
}

impl ScriptFile {
    pub fn new(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content: content.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GeneratorResult {
    pub files: Vec<ScriptFile>,
}

impl GeneratorResult {
    pub fn new(files: Vec<ScriptFile>) -> Self {
        Self { files }
    }
}

impl fmt::Display for GeneratorResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} files generated", self.files.len())
    }
}

#[derive(Clone, Debug, Default)]
pub struct GeneratorSettings {
    pub openapi_path: String,
    pub authorization_header: Option<String>,
    pub content_type: String,
    pub base_url: Option<String>,
    pub generate_bash: bool,
}
