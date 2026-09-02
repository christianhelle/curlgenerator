//! Inputs and outputs of the script generator.

/// The default `Content-Type` used when the caller does not supply one.
pub const DEFAULT_CONTENT_TYPE: &str = "application/json";

/// Settings that control script generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorSettings {
    /// The path or URL of the OpenAPI specification.
    pub open_api_path: String,
    /// The `Authorization` header applied to every request.
    pub authorization_header: Option<String>,
    /// The default `Content-Type` header applied to every request.
    pub content_type: String,
    /// A base URL used when the specification does not declare an absolute server URL.
    pub base_url: Option<String>,
    /// Whether to emit Bash scripts instead of PowerShell scripts.
    pub generate_bash_scripts: bool,
}

impl Default for GeneratorSettings {
    fn default() -> Self {
        Self {
            open_api_path: String::new(),
            authorization_header: None,
            content_type: DEFAULT_CONTENT_TYPE.to_string(),
            base_url: None,
            generate_bash_scripts: false,
        }
    }
}

impl GeneratorSettings {
    /// Creates settings for an OpenAPI path, leaving every other option at its default.
    ///
    /// # Examples
    ///
    /// ```
    /// use curlgenerator_core::GeneratorSettings;
    ///
    /// let settings = GeneratorSettings::new("./openapi.json");
    ///
    /// assert_eq!(settings.content_type, "application/json");
    /// assert!(!settings.generate_bash_scripts);
    /// ```
    pub fn new(open_api_path: impl Into<String>) -> Self {
        Self {
            open_api_path: open_api_path.into(),
            ..Self::default()
        }
    }

    /// Returns the file extension generated scripts are written with.
    pub fn script_extension(&self) -> &'static str {
        if self.generate_bash_scripts {
            "sh"
        } else {
            "ps1"
        }
    }
}

/// A single generated script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptFile {
    /// The file name, including the extension.
    pub filename: String,
    /// The rendered script contents.
    pub content: String,
}

impl ScriptFile {
    /// Creates a script file from a name and its contents.
    pub fn new(filename: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            content: content.into(),
        }
    }
}

/// The result of a generation run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratorResult {
    /// Every generated script, in document order.
    pub files: Vec<ScriptFile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_to_json_powershell_output() {
        let settings = GeneratorSettings::new("./openapi.json");

        assert_eq!(settings.open_api_path, "./openapi.json");
        assert_eq!(settings.content_type, DEFAULT_CONTENT_TYPE);
        assert_eq!(settings.base_url, None);
        assert_eq!(settings.authorization_header, None);
        assert_eq!(settings.script_extension(), "ps1");
    }

    #[test]
    fn bash_settings_use_the_shell_script_extension() {
        let settings = GeneratorSettings {
            generate_bash_scripts: true,
            ..GeneratorSettings::new("./openapi.json")
        };

        assert_eq!(settings.script_extension(), "sh");
    }

    #[test]
    fn script_files_carry_their_name_and_content() {
        let file = ScriptFile::new("GetPet.ps1", "curl");

        assert_eq!(file.filename, "GetPet.ps1");
        assert_eq!(file.content, "curl");
        assert_eq!(GeneratorResult::default().files, Vec::new());
    }
}
