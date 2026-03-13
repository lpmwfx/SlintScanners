//! Event/callback architecture checks — uiux/state-flow.md
//!
//! RULE: UI callbacks delegate to gateway — one call per callback.
//! BANNED: if/else inside callback body.
//! BANNED: Multiple root.x = assignments in one callback.
//! BANNED: Callback body longer than 3 meaningful lines.

use regex::Regex;
use std::sync::LazyLock;

use crate::context::FileContext;
use crate::issue::Issue;

const RULE_BASE: &str = "uiux/state-flow";

static CB_START: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"([\w-]+)\s*(?:\([^)]*\))?\s*=>\s*\{").unwrap()
);
static IF_STMT: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\bif\b").unwrap()
);
static ROOT_ASSIGN: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\broot\.([\w-]+)\s*=[^=]").unwrap()
);
static BRIDGE_CALL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\w+\.\w+\s*\(").unwrap()
);

struct Callback {
    start_line: usize,
    name: String,
    body: Vec<(usize, String)>, // (lineno, content)
}

fn extract_callbacks(lines: &[&str]) -> Vec<Callback> {
    let mut results = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some(cap) = CB_START.captures(lines[i]) {
            let cb_name = cap[1].to_string();
            let start_ln = i + 1;
            let m = cap.get(0).unwrap();
            let after_open = &lines[i][m.end()..];
            let mut depth: i32 = 1 + after_open.matches('{').count() as i32
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

/// Scan for callback body complexity — detect if-statements, multiple root assignments, and lengthy bodies.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    for cb in extract_callbacks(lines) {
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
                format!("callback '{}': if-statement in UI callback \u{2014} move conditional logic to a single AppBridge method", cb.name),
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
            let display: Vec<_> = all_props.iter().take(3).map(|s| s.as_str()).collect();
            let suffix = if all_props.len() > 3 { " ..." } else { "" };
            issues.push(Issue::error(
                ctx.path, first_assign_ln, 1,
                &format!("{}/no-state-mutation-in-callback", RULE_BASE),
                format!(
                    "callback '{}': {} root state mutations ({}{}) \u{2014} replace with a single AppBridge call",
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
                        "callback '{}': mixes root.{} mutation with a bridge call \u{2014} let the bridge own all state changes",
                        cb.name, all_props[0]
                    ),
                ));
            }
        }

        // 3. Too long
        if meaningful.len() > 3 && if_lines.is_empty() && all_props.len() < 2 {
            issues.push(Issue::error(
                ctx.path, cb.start_line, 1,
                &format!("{}/no-callback-logic", RULE_BASE),
                format!(
                    "callback '{}': {}-line body \u{2014} callbacks should be a single delegation call",
                    cb.name, meaningful.len()
                ),
            ));
        }
    }
}
