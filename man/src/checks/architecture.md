# `src/checks/architecture.rs`

## `pub fn check_tree(paths: &[PathBuf]) -> Vec<Issue>`
*Line 72 · fn*

Scan the entire project tree for callbacks delegating to multiple gateways — emit issues if 2+ different gateway objects are used.

---

