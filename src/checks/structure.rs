//! Structure check — global/module-tree.md
//!
//! RULE: One component per file.
//! BANNED: Multiple export component definitions in one .slint file.

use regex::Regex;
use std::sync::LazyLock;

use crate::context::FileContext;
use crate::issue::Issue;

const RULE: &str = "global/module-tree/one-component-per-file";

static COMPONENT_DEF: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"(?m)^\s*(?:export\s+)?component\s+(\w+)").unwrap()
);

/// Scan for multiple component definitions in a single file — emit issues for files with 2+ components.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    let text = lines.join("\n");
    let matches: Vec<_> = COMPONENT_DEF.captures_iter(&text).collect();

    if matches.len() <= 1 {
        return;
    }

    let primary = &matches[0][1];

    // Find line numbers for subsequent components
    for cap in &matches[1..] {
        let Some(m) = cap.get(0) else { continue };
        let name = &cap[1];
        let line = text[..m.start()].matches('\n').count() + 1;

        issues.push(Issue::error(
            ctx.path, line, 1, RULE,
            format!(
                "component '{}' — multiple components in one file (primary: '{}'). \
                 Before extracting: (1) check widgets/ for an existing generic component, \
                 (2) if generic (in property + callback only) → extract to widgets/{}.slint, \
                 (3) if view-specific → extract to a new child file",
                name, primary, name
            ),
        ));
    }
}
