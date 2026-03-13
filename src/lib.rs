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
use walkdir::WalkDir;

/// Walk up from `start` to find a Cargo.toml containing `[workspace]`.
/// Returns the workspace root if found, otherwise the original `start`.
fn find_workspace_root(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    loop {
        let cargo = dir.join("Cargo.toml");
        if cargo.is_file() {
            if let Ok(content) = std::fs::read_to_string(&cargo) {
                if content.contains("[workspace]") {
                    return dir;
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    start.to_path_buf()
}

/// Scan all `.slint` files and emit `cargo:warning` for each violation.
/// Call this from `build.rs`.
///
/// Returns the total number of errors found.
pub fn scan_project() -> usize {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("no cwd"));

    let root = find_workspace_root(&manifest_dir);

    let cfg = Config::load(&root);
    if !cfg.enabled {
        return 0;
    }

    // Collect all .slint file paths for tree-level checks
    let ui_dir = root.join("ui");
    let scan_dir = if ui_dir.is_dir() { ui_dir } else { root.join("src") };

    let slint_files: Vec<_> = WalkDir::new(&scan_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "slint"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut total_errors = 0;

    // Per-file checks
    for path in &slint_files {
        let issues = scan_file(path, &cfg);
        for issue in &issues {
            println!("cargo:warning={}", issue);
            total_errors += 1;
        }
    }

    // Tree-level checks
    if cfg.check_architecture {
        let issues = checks::architecture::check_tree(&slint_files);
        for issue in &issues {
            println!("cargo:warning={}", issue);
            total_errors += 1;
        }
    }

    if total_errors > 0 && cfg.deny {
        panic!(
            "slintscanners: {} error(s) found — fix violations or set deny = false in proj/rulestools.toml",
            total_errors
        );
    }

    total_errors
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

    if cfg.check_tokens {
        checks::tokens::check(&ctx, &lines, &mut issues);
    }
    if cfg.check_strings {
        checks::strings::check(&ctx, &lines, &mut issues);
    }
    if cfg.check_structure {
        checks::structure::check(&ctx, &lines, &mut issues);
    }
    if cfg.check_events {
        checks::events::check(&ctx, &lines, &mut issues);
    }
    if cfg.check_mother_child {
        checks::mother_child::check(&ctx, &lines, &mut issues);
    }
    if cfg.check_string_states {
        checks::string_states::check(&ctx, &lines, &mut issues);
    }

    issues
}
