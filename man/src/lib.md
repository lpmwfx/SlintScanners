# `src/lib.rs`

## `pub mod checks;`
*Line 10 · mod*

Collection of SlintScanners static analysis checks — tokens, strings, structure, events, mother-child, and architecture.

---

## `pub fn scan_project() -> usize`
*Line 45 · fn*

Scan all `.slint` files and emit `cargo:warning` for each violation.
Call this from `build.rs`.

Returns the total number of errors found.

---

## `pub fn scan_file(path: &Path, cfg: &Config) -> Vec<Issue>`
*Line 99 · fn*

Scan a single `.slint` file and return all issues.

---

