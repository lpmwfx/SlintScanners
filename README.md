# SlintScanners

Zero-literal architecture scanner for [Slint](https://slint.dev/) UI files.
Runs during `cargo build` and reports violations as compiler warnings.

Part of the [Rules](https://github.com/lpmwfx/Rules) enforcement toolchain.

## What it checks

| Scanner | Rule | Severity |
|---|---|---|
| **tokens** | No hardcoded colors, px, %, ms, integers, floats in components | error |
| **strings** | No hardcoded string property values | warning |
| **structure** | One component per file | error |
| **events** | No logic in callbacks, max 3 lines, single gateway delegation | error |
| **mother_child** | No `in-out property` in child components, no sibling imports in views/ | error |
| **string_states** | No stringly-typed state comparisons (`== "value"`) | error |
| **architecture** | All callbacks delegate to a single gateway object (tree-level) | error |

### Exempt locations

Files in definition folders are exempt from literal checks:
- `globals/`, `tokens/`, `theme/`, `state/` — these are where values are *defined*

### Exempt syntax

Three Slint constructs require literals and are always exempt:
- `GridLayout` — `row` and `col` properties
- `@image-url("...")` — image paths
- `@tr("...")` — translation strings

## Install

One-line install into any Cargo project that uses Slint:

```bash
curl -sSf https://raw.githubusercontent.com/lpmwfx/SlintScanners/main/install.sh | bash
```

This adds `slintscanners` as a build dependency, patches `build.rs`, and creates a default config.

### Manual install

1. Add to `Cargo.toml`:

```toml
[build-dependencies]
slintscanners = { git = "https://github.com/lpmwfx/SlintScanners" }
```

2. Call from `build.rs`:

```rust
fn main() {
    slintscanners::scan_project();
}
```

## Configuration

All scanners are toggled via `proj/rulestools.toml`:

```toml
[slintscanners]
enabled = true
deny = false        # true = cargo build fails on violations

tokens = true       # hardcoded colors, px, %, ms, int, float
strings = true      # hardcoded string property values
structure = true    # multiple components per file
events = true       # callback logic, state mutations
mother_child = true # in-out property in children, sibling imports
string_states = true # stringly-typed state comparisons
architecture = true # multiple gateway objects

# Optional: exclude paths from scanning (glob patterns)
exclude = ["target/*", "**/vendor/*"]
```

Set `deny = true` to make violations fail the build.

## How it works

`slintscanners::scan_project()` runs at build time:

1. Walks `ui/` (fallback: `src/`) for `.slint` files
2. Runs per-file checks (tokens, strings, structure, events, mother_child, string_states)
3. Runs tree-level check (architecture — single gateway across all files)
4. Emits `cargo:warning=` for each violation

## Related

- [RustScanners](https://github.com/lpmwfx/RustScanners) — zero-literal scanner for Rust source files
- [RulesTools](https://github.com/lpmwfx/RulesTools) — full multi-language scanner (Python, JS, CSS, Rust, Slint, C#, Kotlin)
- [Rules](https://github.com/lpmwfx/Rules) — the coding rules these scanners enforce
