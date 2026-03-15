//! Event/callback architecture checks — uiux/state-flow.md
//!
//! RULE: UI callbacks delegate to gateway — one call per callback.
//! BANNED: if/else inside callback body.
//! BANNED: Multiple root.x = assignments in one callback.
//! BANNED: Callback body longer than 3 meaningful lines.

use regex::Regex;
use std::sync::LazyLock;

use crate::context::{self, FileContext};
use crate::issue::Issue;

const RULE_BASE: &str = "uiux/state-flow";

static IF_STMT: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\bif\b").unwrap()
);
static ROOT_ASSIGN: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\broot\.([\w-]+)\s*=[^=]").unwrap()
);
static BRIDGE_CALL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\w+\.\w+\s*\(").unwrap()
);

const MAX_CALLBACK_LINES: usize = 3;

/// Scan for callback body complexity — detect if-statements, multiple root assignments, and lengthy bodies.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    for cb in context::extract_callbacks(lines) {
        let meaningful: Vec<_> = cb.body.iter()
            .filter(|(_, l)| {
                let t = l.trim();
                !t.is_empty() && !t.starts_with("//")
            })
            .collect();

        // 1. Conditional logic
        let if_lines: Vec<_> = meaningful.iter()
            .filter(|(_, l)| IF_STMT.is_match(l))
            .collect();
        if !if_lines.is_empty() {
            let (ln, l) = if_lines[0];
            let col = l.find("if").unwrap_or(0) + 1;
            issues.push(Issue::error(
                ctx.path, *ln, col,
                &format!("{}/no-callback-logic", RULE_BASE),
                format!("callback '{}': if-statement in UI callback — move conditional logic to a single AppBridge method", cb.name),
            ));
        }

        // 2. Multiple root.x = assignments
        let mut all_props = Vec::new();
        let mut first_assign_ln = 0;
        for (ln, l) in &meaningful {
            let props: Vec<_> = ROOT_ASSIGN.captures_iter(l)
                .map(|c| c[1].to_string())
                .collect();
            if !props.is_empty() && first_assign_ln == 0 {
                first_assign_ln = *ln;
            }
            all_props.extend(props);
        }

        if all_props.len() >= 2 {
            let display: Vec<_> = all_props.iter().take(MAX_CALLBACK_LINES).map(|s| s.as_str()).collect();
            let suffix = if all_props.len() > MAX_CALLBACK_LINES { " ..." } else { "" };
            issues.push(Issue::error(
                ctx.path, first_assign_ln, 1,
                &format!("{}/no-state-mutation-in-callback", RULE_BASE),
                format!(
                    "callback '{}': {} root state mutations ({}{}) — replace with a single AppBridge call",
                    cb.name, all_props.len(), display.join(", "), suffix
                ),
            ));
        } else if all_props.len() == 1 {
            let has_bridge = meaningful.iter().any(|(_, l)| BRIDGE_CALL.is_match(l));
            if has_bridge {
                issues.push(Issue::error(
                    ctx.path, first_assign_ln, 1,
                    &format!("{}/no-state-mutation-in-callback", RULE_BASE),
                    format!(
                        "callback '{}': mixes root.{} mutation with a bridge call — let the bridge own all state changes",
                        cb.name, all_props[0]
                    ),
                ));
            }
        }

        // 3. Too long
        if meaningful.len() > MAX_CALLBACK_LINES && if_lines.is_empty() && all_props.len() < 2 {
            issues.push(Issue::error(
                ctx.path, cb.start_line, 1,
                &format!("{}/no-callback-logic", RULE_BASE),
                format!(
                    "callback '{}': {}-line body — callbacks should be a single delegation call",
                    cb.name, meaningful.len()
                ),
            ));
        }
    }
}
