# `src/checks/mother_child.rs`

## `pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>)`
*Line 29 · fn*

Scan for mother-child architecture violations — detect state ownership in child components, improper in-out properties, and sibling imports.

---

