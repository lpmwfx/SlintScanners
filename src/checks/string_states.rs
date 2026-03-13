//! String state checks — slint/validation.md
//!
//! BANNED: String literals (>=3 chars) used in == comparisons
//!         without central definition in globals/.
//! Exempt: token/globals/theme folders, UI text (spaces, paths, etc.).

use regex::Regex;
use std::sync::LazyLock;

use crate::context::{self, FileContext};
use crate::issue::Issue;

const RULE: &str = "slint/validation/string-state-comparison";

static STR_COMPARE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r#"[!=]=\s*"([^"]{3,})""#).unwrap()
);

static SKIP_VALUES: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^(?:[A-Z]|.*\s.*|.*[/\\].*|.*\.slint|.*\.png|.*\.svg|v\d|\d)").unwrap()
);

/// Scan for string literal comparisons — detect == and != operators comparing string literals to identifiers.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    if ctx.is_definition_file { return; }

    for (idx, raw) in lines.iter().enumerate() {
        let lineno = idx + 1;
        if raw.trim_start().starts_with("//") { continue; }

        let comment_at = context::comment_start(raw);
        let segment = &raw[..comment_at];

        for cap in STR_COMPARE.captures_iter(segment) {
            let val = &cap[1];
            if SKIP_VALUES.is_match(val) { continue; }

            issues.push(Issue::error(
                ctx.path, lineno, cap.get(0).unwrap().start() + 1, RULE,
                format!(
                    "stringly-typed state == \"{}\" \u{2014} all state values must be named constants defined in globals/ or moved to a Rust enum",
                    val
                ),
            ));
        }
    }
}
