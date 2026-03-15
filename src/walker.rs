//! File system walking — collects `.slint` source files for scanning.

use glob::Pattern;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Topology;

const CARGO_TOML: &str = "Cargo.toml";
const SLINT_EXT: &str = "slint";
const WORKSPACE_MAX_DEPTH: usize = 5;
const WORKSPACE_MARKER: &str = "[workspace]";

/// Walk up from `start` to find a `Cargo.toml` containing `[workspace]`.
/// Returns the workspace root if found, otherwise `start`.
pub fn find_workspace_root(start: &Path) -> PathBuf {
    let mut dir = start.to_path_buf();
    loop {
        let cargo = dir.join(CARGO_TOML);
        if cargo.is_file() {
            if let Ok(content) = std::fs::read_to_string(&cargo) {
                if content.contains(WORKSPACE_MARKER) {
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

/// Collect `.slint` files based on declared topology.
///
/// - `Flat`: scans `manifest_dir/ui/` or `manifest_dir/src/` — own crate only.
/// - `Workspace`: walks all workspace member `ui/` and `src/` directories under root.
pub fn collect_slint_files(root: &Path, manifest_dir: &Path, topology: &Topology, exclude: &[String]) -> Vec<PathBuf> {
    match topology {
        Topology::Flat => {
            let ui_dir = manifest_dir.join("ui");
            let scan_dir = if ui_dir.is_dir() { ui_dir } else { manifest_dir.join("src") };
            collect_in_dir(&scan_dir, exclude)
        }
        Topology::Workspace => {
            let root_cargo = root.join(CARGO_TOML);
            let mut files = Vec::new();
            for entry in WalkDir::new(root)
                .max_depth(WORKSPACE_MAX_DEPTH)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name() == CARGO_TOML && e.path() != root_cargo)
            {
                let member = match entry.path().parent() { Some(p) => p, None => continue };
                let ui_dir = member.join("ui");
                let scan_dir = if ui_dir.is_dir() { ui_dir } else { member.join("src") };
                if scan_dir.is_dir() {
                    files.extend(collect_in_dir(&scan_dir, exclude));
                }
            }
            files
        }
    }
}

fn collect_in_dir(dir: &Path, exclude: &[String]) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !is_excluded(e.path(), exclude))
        .filter(|e| e.path().extension().map_or(false, |ext| ext == SLINT_EXT))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| Pattern::new(p).map_or(false, |pat| pat.matches(&path_str)))
}
