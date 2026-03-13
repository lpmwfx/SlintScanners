//! Mother-child architecture checks — uiux/mother-child.md
//!
//! VITAL: Only the mother (inherits Window) may own state.
//! BANNED: in-out property in child components (except <=> delegation).
//! BANNED: View importing a sibling view (views/ folder).

use regex::Regex;
use std::sync::LazyLock;

use crate::context::FileContext;
use crate::issue::Issue;

const RULE_BASE: &str = "uiux/mother-child";

static IN_OUT_PROP: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^\s*in-out\s+property\b").unwrap()
);
static DELEGATION: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"<=>").unwrap()
);
static IMPORT: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r#"import\s+\{[^}]*\}\s+from\s+"([^"]+)""#).unwrap()
);
static STD_IMPORT: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^std-widgets\.slint$").unwrap()
);

/// Scan for mother-child architecture violations — detect state ownership in child components, improper in-out properties, and sibling imports.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    // Mother is allowed to own state
    if ctx.is_mother { return; }
    // Global definition files are exempt
    if ctx.is_global_file { return; }

    // 1. Child has state (in-out property without <=>)
    for (idx, raw) in lines.iter().enumerate() {
        let lineno = idx + 1;
        if raw.trim_start().starts_with("//") { continue; }

        if IN_OUT_PROP.is_match(raw) && !DELEGATION.is_match(raw) {
            issues.push(Issue::error(
                ctx.path, lineno, 1,
                &format!("{}/child-has-state", RULE_BASE),
                "in-out property in child component \u{2014} children must be stateless. \
                 Use 'in property' to receive state from mother, 'callback' to emit events up."
                    .to_string(),
            ));
        }
    }

    // 2. Sibling import in views/
    if ctx.is_views_folder {
        for (idx, raw) in lines.iter().enumerate() {
            let lineno = idx + 1;
            if let Some(cap) = IMPORT.captures(raw) {
                let import_path = &cap[1];
                if STD_IMPORT.is_match(import_path) { continue; }
                if import_path.starts_with("../") || import_path.starts_with("./") { continue; }

                issues.push(Issue::error(
                    ctx.path, lineno, cap.get(1).unwrap().start() + 1,
                    &format!("{}/sibling-import", RULE_BASE),
                    format!(
                        "view imports sibling view '{}' \u{2014} views must not know about each other. \
                         Route shared state through mother instead.",
                        import_path
                    ),
                ));
            }
        }
    }
}
