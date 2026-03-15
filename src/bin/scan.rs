//! `slintscanners` — standalone CLI scanner for Slint UI files.
//!
//! Usage: slintscanners [PATH]
//!
//! Scans the project at PATH (or current directory) and prints violations
//! in `file:line:col: error rule: message` format.
//! Exits 1 if any issues are found.

use std::path::PathBuf;

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("no cwd"));

    let issues = slintscanners::scan_at(&root);
    for iss in &issues {
        println!("{}", iss);
    }
    if !issues.is_empty() {
        std::process::exit(1);
    }
}
