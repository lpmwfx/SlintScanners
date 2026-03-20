# `src/issue.rs`

## `pub enum Severity`
*Line 6 · enum*

Severity level of a scanner issue — currently only Error is used.

---

## `pub struct Issue`
*Line 12 · struct*

A single violation found by the scanner — file, line, column, rule ID, and message.

---

## `pub fn error(file: &Path, line: usize, col: usize, rule: &str, message: String) -> Self`
*Line 23 · fn*

Create an Error-severity issue at the given file location with rule ID and message.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=1" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
