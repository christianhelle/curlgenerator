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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_file_new() {
        let file = ScriptFile::new("test.ps1".to_string(), "curl -X GET".to_string());
        assert_eq!(file.filename, "test.ps1");
        assert_eq!(file.content, "curl -X GET");
    }

    #[test]
    fn test_script_file_clone() {
        let file = ScriptFile::new("test.sh".to_string(), "#!/bin/bash".to_string());
        let cloned = file.clone();
        assert_eq!(file.filename, cloned.filename);
        assert_eq!(file.content, cloned.content);
    }

    #[test]
    fn test_script_file_with_empty_strings() {
        let file = ScriptFile::new(String::new(), String::new());
        assert_eq!(file.filename, "");
        assert_eq!(file.content, "");
    }

    #[test]
    fn test_script_file_with_special_characters() {
        let file = ScriptFile::new(
            "special-file_123.ps1".to_string(),
            "curl -H 'Authorization: Bearer $token'".to_string(),
        );
        assert_eq!(file.filename, "special-file_123.ps1");
        assert!(file.content.contains("Bearer"));
    }
}
