//! Architecture check — uiux/state-flow.md
//!
//! RULE: All UI callbacks delegate to exactly ONE gateway object.
//! Tree-level check: scans all .slint files then reports if callbacks
//! use multiple different gateway objects.

use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::context;
use crate::issue::Issue;

const RULE: &str = "uiux/state-flow/single-gateway";

static BRIDGE_CALL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b([A-Z][A-Za-z0-9]*)\.([\w-]+)\s*\(").unwrap()
);

const BUILTIN_RECEIVERS: &[&str] = &[
    "Math", "Colors", "Palette", "Theme", "StyleMetrics", "TextInputInterface",
];

/// Scan the entire project tree for callbacks delegating to multiple gateways — emit issues if 2+ different gateway objects are used.
pub fn check_tree(paths: &[PathBuf]) -> Vec<Issue> {
    let mut all_receivers: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    for path in paths {
        if path.extension().map_or(true, |e| e != "slint") { continue; }
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();

        for cb in context::extract_callbacks(&lines) {
            for (lineno, line) in &cb.body {
                for cap in BRIDGE_CALL.captures_iter(line) {
                    let receiver = &cap[1];
                    if !BUILTIN_RECEIVERS.contains(&receiver) {
                        all_receivers
                            .entry(receiver.to_string())
                            .or_default()
                            .push((path.clone(), *lineno));
                    }
                }
            }
        }
    }

    if all_receivers.len() <= 1 {
        return vec![];
    }

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
                    "callback calls '{}' but the gateway is '{}' — all UI callbacks must delegate through the single gateway object",
                    receiver, dominant
                ),
            ));
        }
    }

    issues
}
