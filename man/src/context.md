# `src/context.rs`

## `pub struct FileContext<'a>`
*Line 4 · struct*

Shared context for a .slint file being scanned.

---

## `pub fn new(path: &'a Path, lines: &[&str]) -> Self`
*Line 17 · fn*

Create a new FileContext for a .slint file being scanned, detecting its role and folder.

---

## `pub fn comment_start(line: &str) -> usize`
*Line 50 · fn*

Find comment start position on a line.

---

