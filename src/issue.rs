use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule: String,
    pub message: String,
}

impl Issue {
    pub fn error(file: &Path, line: usize, col: usize, rule: &str, message: String) -> Self {
        Self {
            file: file.to_path_buf(),
            line,
            col,
            severity: Severity::Error,
            rule: rule.to_string(),
            message,
        }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: error {}: {}",
            self.file.display(),
            self.line,
            self.col,
            self.rule,
            self.message
        )
    }
}
