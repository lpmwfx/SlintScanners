//! Zero-literal enforcement for Slint — slint/states.md + uiux/tokens.md
//!
//! BANNED in component files:
//!   - Hardcoded hex colors (#rgb, #rrggbb, #rrggbbaa)
//!   - rgb()/rgba() calls
//!   - Pixel sizes (ALL, including 0px)
//!   - Percentages (ALL)
//!   - Durations (ALL ms/s values)
//!   - Bare integers and floats
//!
//! Exempt: globals/, tokens/, theme/, state/ folders.
//! Syntax exceptions: GridLayout row/col, @image-url, @tr.

use regex::Regex;
use std::sync::LazyLock;

use crate::context::{self, FileContext};
use crate::issue::Issue;

const RULE_COLORS: &str = "uiux/tokens/no-hardcoded-color";
const RULE_SIZES: &str = "slint/states/no-hardcoded-size";
const RULE_DURATION: &str = "slint/states/no-hardcoded-duration";
const RULE_NUMBER: &str = "slint/states/no-hardcoded-number";

static HEX_COLOR: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"#(?:[0-9a-fA-F]{3,4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})\b").unwrap()
);
static RGB_FUNC: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\brgba?\s*\(").unwrap()
);
static PIXEL_VALUE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b\d+(?:\.\d+)?\s*px\b").unwrap()
);
static PERCENT_VALUE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b\d+(?:\.\d+)?\s*%").unwrap()
);
static DURATION_VALUE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b\d+(?:\.\d+)?\s*(?:ms|s)\b").unwrap()
);
static FLOAT_VALUE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b(\d+\.\d+)\b").unwrap()
);
static INT_VALUE: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b(\d+)\b").unwrap()
);
// Units that claim numbers — if a number is followed by a unit it's already
// caught by the px/percent/duration checks.
static UNIT_SUFFIX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"^\s*(?:px|%|ms|s)\b").unwrap()
);

// Syntax exceptions
static GRID_ROW_COL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\b(?:row|col)\s*:\s*\d+").unwrap()
);
static IMAGE_URL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"@image-url\s*\(").unwrap()
);
static TR_MACRO: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"@tr\s*\(").unwrap()
);
static PROPERTY_DECL: LazyLock<Regex> = LazyLock::new(||
    Regex::new(r"\bproperty\s*<").unwrap()
);

fn is_property_decl(line: &str, match_start: usize) -> bool {
    PROPERTY_DECL.is_match(&line[..match_start])
}

/// Scan for hardcoded zero-literals — detect colors, sizes, durations, and plain numbers in component definitions.
pub fn check(ctx: &FileContext, lines: &[&str], issues: &mut Vec<Issue>) {
    if ctx.is_definition_file {
        return;
    }

    for (idx, raw) in lines.iter().enumerate() {
        let lineno = idx + 1;
        let stripped = raw.trim_start();

        if stripped.starts_with("//") {
            continue;
        }

        let comment_at = context::comment_start(raw);
        let segment = &raw[..comment_at];

        // Syntax exceptions
        if IMAGE_URL.is_match(segment) || TR_MACRO.is_match(segment) {
            continue;
        }

        // Hex colors
        for m in HEX_COLOR.find_iter(segment) {
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_COLORS,
                format!("hardcoded color '{}' \u{2014} use Colors.* token", m.as_str()),
            ));
        }

        // rgb/rgba
        for m in RGB_FUNC.find_iter(segment) {
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_COLORS,
                "hardcoded rgb/rgba() \u{2014} use Colors.* token".to_string(),
            ));
        }

        // Pixel sizes
        for m in PIXEL_VALUE.find_iter(segment) {
            if is_property_decl(raw, m.start()) { continue; }
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_SIZES,
                format!("hardcoded size '{}' \u{2014} use Sizes.* or Spacing.* variable", m.as_str().trim()),
            ));
        }

        // Percentages
        for m in PERCENT_VALUE.find_iter(segment) {
            if is_property_decl(raw, m.start()) { continue; }
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_SIZES,
                format!("hardcoded percentage '{}' \u{2014} use Sizes.full / Sizes.half variable", m.as_str().trim()),
            ));
        }

        // Durations
        for m in DURATION_VALUE.find_iter(segment) {
            if is_property_decl(raw, m.start()) { continue; }
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_DURATION,
                format!("hardcoded duration '{}' \u{2014} use Durations.* variable", m.as_str().trim()),
            ));
        }

        // Collect positions already covered by px/percent/duration checks
        let mut covered: Vec<(usize, usize)> = Vec::new();
        for m in PIXEL_VALUE.find_iter(segment) { covered.push((m.start(), m.end())); }
        for m in PERCENT_VALUE.find_iter(segment) { covered.push((m.start(), m.end())); }
        for m in DURATION_VALUE.find_iter(segment) { covered.push((m.start(), m.end())); }
        for m in HEX_COLOR.find_iter(segment) { covered.push((m.start(), m.end())); }

        let is_covered = |start: usize, end: usize| -> bool {
            covered.iter().any(|&(cs, ce)| start >= cs && end <= ce)
        };

        // Floats (not already caught by px/duration/percent)
        for cap in FLOAT_VALUE.captures_iter(segment) {
            let m = cap.get(1).unwrap();
            if is_covered(m.start(), m.end()) { continue; }
            if is_property_decl(raw, m.start()) { continue; }
            // Skip if preceded by # (hex)
            if m.start() > 0 && segment.as_bytes()[m.start() - 1] == b'#' { continue; }
            // Skip if followed by unit
            if UNIT_SUFFIX.is_match(&segment[m.end()..]) { continue; }
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_NUMBER,
                format!("hardcoded float '{}' \u{2014} use state variable or theme token", m.as_str()),
            ));
        }

        // Integers (not already caught by other checks)
        for cap in INT_VALUE.captures_iter(segment) {
            let m = cap.get(1).unwrap();
            if is_covered(m.start(), m.end()) { continue; }
            if is_property_decl(raw, m.start()) { continue; }
            if GRID_ROW_COL.is_match(raw) { continue; }
            // Skip if preceded by # (hex) or . (field) or letter/underscore (identifier)
            if m.start() > 0 {
                let prev = segment.as_bytes()[m.start() - 1];
                if prev == b'#' || prev == b'.' || prev == b'_'
                    || prev.is_ascii_alphabetic()
                {
                    continue;
                }
            }
            // Skip if followed by . (decimal) or letter (identifier suffix) or unit
            if m.end() < segment.len() {
                let rest = &segment[m.end()..];
                if rest.starts_with('.') || rest.as_bytes()[0].is_ascii_alphabetic()
                    || rest.as_bytes()[0] == b'_'
                {
                    continue;
                }
            }
            // Skip if followed by unit
            if UNIT_SUFFIX.is_match(&segment[m.end()..]) { continue; }
            issues.push(Issue::error(
                ctx.path, lineno, m.start() + 1, RULE_NUMBER,
                format!("hardcoded integer '{}' \u{2014} use state variable (ViewStates.*, Sizes.*)", m.as_str()),
            ));
        }
    }
}
