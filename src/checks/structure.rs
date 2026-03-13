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

pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    let text = lines.join("\n");
    let matches: Vec<_> = COMPONENT_DEF.captures_iter(&text).collect();

    if matches.len() < 2 {
        return;
    }

    let primary = &matches[0][1];

    // Find line numbers for subsequent components
    for cap in &matches[1..] {
        let m = cap.get(0).unwrap();
        let name = &cap[1];
        let line = text[..m.start()].matches('\n').count() + 1;

        issues.push(Issue::error(
            ctx.path, line, 1, RULE,
            format!(
                "component '{}' \u{2014} multiple components in one file. Extract to '{}.slint' (primary: '{}')",
                name, name, primary
            ),
        ));
    }
}
