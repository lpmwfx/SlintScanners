use std::path::Path;

/// Shared context for a .slint file being scanned.
pub struct FileContext<'a> {
    pub path: &'a Path,
    pub is_definition_file: bool,
    pub is_mother: bool,
    pub is_global_file: bool,
    pub is_views_folder: bool,
}

/// Definition folders — exempt from token/literal checks.
const DEFINITION_FOLDERS: &[&str] = &["globals", "tokens", "theme", "state"];

impl<'a> FileContext<'a> {
    pub fn new(path: &'a Path, lines: &[&str]) -> Self {
        Self {
            path,
            is_definition_file: is_definition_file(path),
            is_mother: is_mother(lines),
            is_global_file: is_global_file(lines),
            is_views_folder: is_views_folder(path),
        }
    }
}

fn is_definition_file(path: &Path) -> bool {
    path.components()
        .any(|c| DEFINITION_FOLDERS.contains(&c.as_os_str().to_str().unwrap_or("")))
}

fn is_mother(lines: &[&str]) -> bool {
    lines.iter().any(|l| l.contains("inherits Window"))
}

fn is_global_file(lines: &[&str]) -> bool {
    lines.iter().take(40).any(|l| {
        let t = l.trim_start();
        t.starts_with("export global ")
    })
}

fn is_views_folder(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str().to_str() == Some("views"))
}

/// Find comment start position on a line.
pub fn comment_start(line: &str) -> usize {
    line.find("//").unwrap_or(line.len())
}
