# `src/checks/architecture.rs`

## `pub fn check_tree(paths: &[PathBuf]) -> Vec<Issue>`
*Line 72 · fn*

Scan the entire project tree for callbacks delegating to multiple gateways — emit issues if 2+ different gateway objects are used.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=1" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
