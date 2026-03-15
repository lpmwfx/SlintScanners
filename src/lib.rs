//! # SlintScanners
//!
//! Zero-literal static analysis scanners for Slint UI files.
//! Add as a `[build-dependencies]` and call `slintscanners::scan_project()`
//! from `build.rs` to enforce tokens/themes/states during `cargo build`.
//!
//! Separate from RustScanners — only install this in Slint projects.

/// Collection of SlintScanners static analysis checks — tokens, strings, structure, events, mother-child, and architecture.
pub mod checks;
mod config;
mod issue;
mod context;

pub use config::Config;
pub use issue::Issue;

use std::path::{Path, PathBuf};

mod walker;

// ── Check dispatch ────────────────────────────────────────────────────────────

type CheckFn = fn(&context::FileContext, &[&str], &mut Vec<Issue>);

/// Per-file check registry — add one line here to register a new check.
/// Each entry: (enabled predicate on Config, check function).
const FILE_CHECKS: &[(fn(&Config) -> bool, CheckFn)] = &[
    (|c| c.check_tokens,        checks::tokens::check),
    (|c| c.check_strings,       checks::strings::check),
    (|c| c.check_structure,     checks::structure::check),
    (|c| c.check_events,        checks::events::check),
    (|c| c.check_mother_child,  checks::mother_child::check),
    (|c| c.check_string_states, checks::string_states::check),
];

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Emit issues as `cargo:warning=` lines and return the count.
fn emit_issues(issues: &[Issue]) -> usize {
    for issue in issues {
        println!("cargo:warning={}", issue);
    }
    issues.len()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Scan all `.slint` files and emit `cargo:warning` for each violation.
/// Call this from `build.rs`.
///
/// Returns the total number of errors found.
pub fn scan_project() -> usize {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("no cwd"));

    let root = walker::find_workspace_root(&manifest_dir);

    let cfg = Config::load(&root, &manifest_dir);
    if !cfg.enabled {
        return 0;
    }

    let slint_files = walker::collect_slint_files(&root, &manifest_dir, &cfg.topology, &cfg.exclude);

    let mut total_errors = 0;

    for path in &slint_files {
        total_errors += emit_issues(&scan_file(path, &cfg));
    }

    if cfg.check_architecture {
        total_errors += emit_issues(&checks::architecture::check_tree(&slint_files));
    }

    if total_errors > 0 && cfg.deny {
        panic!(
            "slintscanners: {} error(s) found — fix violations or set deny = false in proj/rulestools.toml",
            total_errors
        );
    }

    total_errors
}

/// Scan all `.slint` files at `root` and return all issues.
///
/// Unlike `scan_project()`, this does not use `CARGO_MANIFEST_DIR` —
/// suitable for standalone CLI use.
pub fn scan_at(root: &Path) -> Vec<Issue> {
    let cfg = Config::load(root, root);
    if !cfg.enabled {
        return vec![];
    }
    let slint_files = walker::collect_slint_files(root, root, &cfg.topology, &cfg.exclude);
    let mut issues: Vec<Issue> = Vec::new();
    for path in &slint_files {
        issues.extend(scan_file(path, &cfg));
    }
    if cfg.check_architecture {
        issues.extend(checks::architecture::check_tree(&slint_files));
    }
    issues
}

/// Scan a single `.slint` file and return all issues.
pub fn scan_file(path: &Path, cfg: &Config) -> Vec<Issue> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let lines: Vec<&str> = content.lines().collect();
    let ctx = context::FileContext::new(path, &lines);
    let mut issues = Vec::new();

    for (enabled, check) in FILE_CHECKS {
        if enabled(cfg) {
            check(&ctx, &lines, &mut issues);
        }
    }

    issues
}
