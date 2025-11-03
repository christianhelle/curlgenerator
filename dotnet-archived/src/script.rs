#[derive(Debug, Clone)]
pub struct ScriptFile {
    pub filename: String,
    pub content: String,
}

impl ScriptFile {
    pub fn new(filename: String, content: String) -> Self {
        Self { filename, content }
    }
}
