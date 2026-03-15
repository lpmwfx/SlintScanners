//! Hardcoded string check — slint/validation.md
//!
//! BANNED: String literals used directly as property values in component body.
//! Exempt: single-char strings, empty strings, allowed props (icon, source, etc.).

use regex::Regex;
use std::sync::LazyLock;

use crate::context::{self, FileContext};
use crate::issue::Issue;

const RULE: &str = "slint/validation/no-hardcoded-string";

static HARDCODED_STR: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r#"^\s*([\w-]+)\s*:\s*"([^"]{2,})""#).unwrap()
);

static ALLOWED_PATTERNS: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^(@|#|/|\\|\d|\{|\[)").unwrap()
);

const ALLOWED_PROPS: &[&str] = &[
    "accessible-label", "icon", "source", "background-image",
];

/// Scan for hardcoded string literals in property values — detect string assignments to text, title, and similar properties.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    let _ = ctx; // no special context needed

    for (lineno, raw, _) in context::code_lines(lines) {
        if let Some(cap) = HARDCODED_STR.captures(raw) {
            let prop = cap[1].to_lowercase();
            let value = &cap[2];

            if ALLOWED_PROPS.iter().any(|&p| p == prop) { continue; }
            if ALLOWED_PATTERNS.is_match(value) { continue; }
            // Skip property declarations
            if let Some(quote_pos) = raw.find('"') {
                if raw[..quote_pos].contains("property") { continue; }
            }

            let col = raw.find('"').unwrap_or(0) + 1;
            issues.push(Issue::error(
                ctx.path, lineno, col, RULE,
                format!(
                    "hardcoded string '{}' in component \u{2014} expose as an `in property <string> {}` so callers can configure it",
                    value, prop
                ),
            ));
        }
    }
}
