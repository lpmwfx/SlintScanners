use std::path::Path;
use regex::Regex;
use std::sync::LazyLock;

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
    /// Create a new FileContext for a .slint file being scanned, detecting its role and folder.
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

/// Iterate meaningful lines — skips full-line comments, yields `(lineno, raw, segment)`
/// where `segment` is `raw` with any inline comment stripped.
/// Use `_` for elements not needed: `for (lineno, raw, _) in code_lines(lines)`.
pub fn code_lines<'a>(
    lines: &'a [&'a str],
) -> impl Iterator<Item = (usize, &'a str, &'a str)> + 'a {
    lines
        .iter()
        .enumerate()
        .filter(|(_, raw)| !raw.trim_start().starts_with("//"))
        .map(|(idx, raw)| {
            let comment_at = comment_start(raw);
            (idx + 1, *raw, &raw[..comment_at])
        })
}

// ── Callback extraction ───────────────────────────────────────────────────────

/// A parsed callback/event handler block from a .slint file.
pub struct Callback {
    pub start_line: usize,
    pub name: String,
    pub body: Vec<(usize, String)>, // (lineno, line content)
}

static CB_START: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"([\w-]+)\s*(?:\([^)]*\))?\s*=>\s*\{").unwrap()
);

/// Extract all callback blocks from a .slint file's lines.
/// Shared between `checks::events` (body complexity) and `checks::architecture` (gateway detection).
pub fn extract_callbacks(lines: &[&str]) -> Vec<Callback> {
    let mut results = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some(cap) = CB_START.captures(lines[i]) {
            let cb_name = cap[1].to_string();
            let start_ln = i + 1;
            let Some(m) = cap.get(0) else { i += 1; continue };
            let after_open = &lines[i][m.end()..];
            let mut depth: i32 = 1
                + after_open.matches('{').count() as i32
                - after_open.matches('}').count() as i32;

            if depth <= 0 {
                results.push(Callback {
                    start_line: start_ln,
                    name: cb_name,
                    body: vec![(i + 1, lines[i].to_string())],
                });
                i += 1;
                continue;
            }

            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && depth > 0 {
                depth += lines[i].matches('{').count() as i32
                    - lines[i].matches('}').count() as i32;
                if depth > 0 {
                    body.push((i + 1, lines[i].to_string()));
                }
                i += 1;
            }
            results.push(Callback { start_line: start_ln, name: cb_name, body });
            continue;
        }
        i += 1;
    }
    results
}
