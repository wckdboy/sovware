#!/usr/bin/env bash
# CI and pre-merge check. No network, no model, no credentials.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "▪ fmt"
cargo fmt --check

echo "▪ clippy"
cargo clippy --all-targets -- -D warnings

echo "▪ tests"
cargo test --quiet

echo "▪ schemas are valid json"
for f in schema/*.json; do
  python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$f"
  echo "  ✓ $f"
done

echo "▪ loop"
cargo build --quiet
SOV=./target/debug/sov ./scripts/demo.sh
