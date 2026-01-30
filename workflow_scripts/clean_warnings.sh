#!/usr/bin/env bash
set -euo pipefail

echo "== Rust Workspace Warning Cleanup Script =="

# Ensure we're in the workspace root
if [ ! -f Cargo.toml ]; then
  echo "ERROR: Run this from workspace root (where Cargo.toml is)"
  exit 1
fi

echo ">> Formatting code..."
cargo fmt --all

echo ">> Running cargo fix..."
cargo fix --workspace --all-targets --allow-dirty --allow-staged

echo ">> Running Clippy (workspace)..."
cargo clippy --workspace --all-targets -- -D warnings

echo ">> Running tests (optional)..."
cargo test --workspace

echo "=========================================="
echo "SUCCESS: No warnings left in workspace 🎉"
