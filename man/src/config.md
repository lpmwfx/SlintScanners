# `src/config.rs`

## `pub struct Config`
*Line 6 · struct*

Scanner configuration — controls which checks are enabled and whether violations block the build.

---

## `pub fn load(project_root: &Path) -> Self`
*Line 55 · fn*

Load configuration from `proj/rulestools.toml` under `[slintscanners]` section, or return defaults if not found.

---



---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=1" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
