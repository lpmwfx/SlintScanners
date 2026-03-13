//! Architecture check — uiux/state-flow.md
//!
//! RULE: All UI callbacks delegate to exactly ONE gateway object.
//! Tree-level check: scans all .slint files then reports if callbacks
//! use multiple different gateway objects.

use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use crate::issue::Issue;

const RULE: &str = "uiux/state-flow/single-gateway";

static CB_START: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"[\w-]+\s*(?:\([^)]*\))?\s*=>\s*\{").unwrap()
);
static BRIDGE_CALL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b([A-Z][A-Za-z0-9]*)\.([\w-]+)\s*\(").unwrap()
);

const BUILTIN_RECEIVERS: &[&str] = &[
    "Math", "Colors", "Palette", "Theme", "StyleMetrics", "TextInputInterface",
];

fn collect_bridge_calls(path: &Path) -> Vec<(usize, String)> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut results = Vec::new();
    let mut in_callback = false;
    let mut depth: i32 = 0;

    for (idx, raw) in lines.iter().enumerate() {
        let lineno = idx + 1;

        if !in_callback {
            if CB_START.is_match(raw) {
                in_callback = true;
                depth = raw.matches('{').count() as i32 - raw.matches('}').count() as i32;
                if depth <= 0 {
                    in_callback = false;
                    continue;
                }
            }
        } else {
            depth += raw.matches('{').count() as i32 - raw.matches('}').count() as i32;
            if depth <= 0 {
                in_callback = false;
                continue;
            }
        }

        if in_callback {
            for cap in BRIDGE_CALL.captures_iter(raw) {
                let receiver = &cap[1];
                if !BUILTIN_RECEIVERS.contains(&receiver) {
                    results.push((lineno, receiver.to_string()));
                }
            }
        }
    }

    results
}

/// Scan the entire project tree for callbacks delegating to multiple gateways — emit issues if 2+ different gateway objects are used.
pub fn check_tree(paths: &[PathBuf]) -> Vec<Issue> {
    let mut all_receivers: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    for path in paths {
        if path.extension().map_or(true, |e| e != "slint") { continue; }
        for (lineno, receiver) in collect_bridge_calls(path) {
            all_receivers.entry(receiver)
                .or_default()
                .push((path.clone(), lineno));
        }
    }

    if all_receivers.len() <= 1 {
        return vec![];
    }

    // Dominant = most used receiver
    let dominant = all_receivers.iter()
        .max_by_key(|(_, locs)| locs.len())
        .map(|(name, _)| name.clone())
        .unwrap_or_default();

    let mut issues = Vec::new();
    for (receiver, locations) in &all_receivers {
        if *receiver == dominant { continue; }
        for (path, lineno) in locations {
            issues.push(Issue::error(
                path, *lineno, 1, RULE,
                format!(
                    "callback calls '{}' but the gateway is '{}' \u{2014} all UI callbacks must delegate through the single gateway object",
                    receiver, dominant
                ),
            ));
        }
    }

    issues
}
