use std::fmt;
use std::path::{Path, PathBuf};

/// Severity level of a scanner issue — currently only Error is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
}

/// A single violation found by the scanner — file, line, column, rule ID, and message.
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
    /// Create an Error-severity issue at the given file location with rule ID and message.
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
