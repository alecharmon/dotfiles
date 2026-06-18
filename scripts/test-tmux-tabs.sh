#!/usr/bin/env bash
# Run the tmux-tabs Rust test suite (unit + integration + e2e).
# The e2e tests spin up an isolated tmux server, so tmux must be installed.
set -euo pipefail

DOTFILES_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CRATE_DIR="$DOTFILES_DIR/tmux-tabs"

if ! command -v cargo >/dev/null 2>&1; then
    echo "test-tmux-tabs: cargo not found; install Rust to run these tests." >&2
    exit 1
fi
if ! command -v tmux >/dev/null 2>&1; then
    echo "test-tmux-tabs: tmux not found; the e2e tests require it." >&2
    exit 1
fi

# e2e tests each run their own tmux server on a unique socket; serialise to keep
# output readable and avoid contending for the same default config.
cargo test --manifest-path "$CRATE_DIR/Cargo.toml" -- --test-threads=1

echo "tmux-tabs tests passed"
