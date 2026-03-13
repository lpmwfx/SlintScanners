#!/usr/bin/env bash
set -euo pipefail

# SlintScanners installer — adds Slint zero-literal scanning to cargo build.
#
# Usage:  curl -sSf https://raw.githubusercontent.com/lpmwfx/SlintScanners/main/install.sh | bash
#
# What it does:
#   1. Adds slintscanners as a [build-dependencies] entry in Cargo.toml
#   2. Creates or patches build.rs to call slintscanners::scan_project()
#   3. Creates proj/rulestools.toml with default config (if missing)

CRATE_REF='slintscanners = { git = "https://github.com/lpmwfx/SlintScanners" }'

echo "=== SlintScanners installer ==="

# ── Must be in a Cargo project ──────────────────────────────────────────────
if [ ! -f Cargo.toml ]; then
    echo "ERROR: No Cargo.toml found. Run this from your Rust project root."
    exit 1
fi

# ── 1. Add [build-dependencies] ─────────────────────────────────────────────
if grep -q 'slintscanners' Cargo.toml; then
    echo "[ok] slintscanners already in Cargo.toml"
else
    if grep -q '\[build-dependencies\]' Cargo.toml; then
        sed -i '/\[build-dependencies\]/a '"$CRATE_REF" Cargo.toml
        echo "[+] Added slintscanners to existing [build-dependencies]"
    else
        printf '\n[build-dependencies]\n%s\n' "$CRATE_REF" >> Cargo.toml
        echo "[+] Added [build-dependencies] with slintscanners"
    fi
fi

# ── 2. Create or patch build.rs ──────────────────────────────────────────────
SCAN_LINE="    slintscanners::scan_project();"
BUILD_RS_TEMPLATE='fn main() {
    slintscanners::scan_project();
}'

if [ ! -f build.rs ]; then
    echo "$BUILD_RS_TEMPLATE" > build.rs
    echo "[+] Created build.rs"
elif grep -q 'slintscanners' build.rs; then
    echo "[ok] build.rs already calls slintscanners"
else
    # Insert scan_project() as first line inside fn main()
    sed -i '/fn main()/a\'"$SCAN_LINE" build.rs
    echo "[+] Patched build.rs — added slintscanners::scan_project()"
fi

# ── 3. Create default config ────────────────────────────────────────────────
mkdir -p proj
if [ ! -f proj/rulestools.toml ]; then
    cat > proj/rulestools.toml << 'EOF'
[slintscanners]
enabled = true
deny = false                  # true = cargo build fails on violations

# Toggle individual scanners
tokens = true                 # hardcoded colors, px, %, ms, int, float
strings = true                # hardcoded string property values
structure = true              # multiple components per file
events = true                 # callback logic, state mutations
mother_child = true           # in-out property in children, sibling imports
string_states = true          # stringly-typed state comparisons
architecture = true           # multiple gateway objects
EOF
    echo "[+] Created proj/rulestools.toml"
elif ! grep -q '\[slintscanners\]' proj/rulestools.toml; then
    cat >> proj/rulestools.toml << 'EOF'

[slintscanners]
enabled = true
deny = false                  # true = cargo build fails on violations

# Toggle individual scanners
tokens = true                 # hardcoded colors, px, %, ms, int, float
strings = true                # hardcoded string property values
structure = true              # multiple components per file
events = true                 # callback logic, state mutations
mother_child = true           # in-out property in children, sibling imports
string_states = true          # stringly-typed state comparisons
architecture = true           # multiple gateway objects
EOF
    echo "[+] Added [slintscanners] to existing proj/rulestools.toml"
else
    echo "[ok] proj/rulestools.toml already has [slintscanners]"
fi

echo ""
echo "Done! Run 'cargo build' to see scanner output."
echo "Configure scanners in proj/rulestools.toml"
echo "Set deny = true to make violations break the build."
