# `src/checks/string_states.rs`

## `pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>)`
*Line 24 · fn*

Scan for string literal comparisons — detect == and != operators comparing string literals to identifiers.

---

