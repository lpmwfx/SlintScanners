//! # SlintScanners
//!
//! Zero-literal static analysis scanners for Slint UI files.
//! Add as a `[build-dependencies]` and call `slintscanners::scan_project()`
//! from `build.rs` to enforce tokens/themes/states during `cargo build`.
//!
//! Separate from RustScanners — only install this in Slint projects.

pub mod checks;
mod config;
mod issue;
mod context;

pub use config::Config;
pub use issue::{Issue, Severity};

use std::path::Path;
use walkdir::WalkDir;

/// Scan all `.slint` files and emit `cargo:warning` for each violation.
/// Call this from `build.rs`.
///
/// Returns the total number of errors found.
pub fn scan_project() -> usize {
    let root = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().expect("no cwd"));

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
            if issue.severity == Severity::Error {
                total_errors += 1;
            }
        }
    }

    // Tree-level checks
    if cfg.check_architecture {
        let issues = checks::architecture::check_tree(&slint_files);
        for issue in &issues {
            println!("cargo:warning={}", issue);
            if issue.severity == Severity::Error {
                total_errors += 1;
            }
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
