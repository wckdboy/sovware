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

echo "▪ eval"
./target/debug/sov eval examples/bracket/eval.json \
  --ledger .demo/runs.jsonl \
  --report .demo/eval-report.json

echo "▪ a tampered holdout is refused"
mkdir -p .demo/tamper
cp examples/bracket/*.json examples/bracket/bracket.py .demo/tamper/
python3 - <<'PY'
import json
path = ".demo/tamper/eval.json"
data = json.load(open(path))
data["holdout"][0]["note"] = "edited after the set was frozen"
json.dump(data, open(path, "w"), indent=2)
PY
if ./target/debug/sov eval .demo/tamper/eval.json --ledger .demo/runs.jsonl >/dev/null 2>&1; then
  echo "✗ a tampered holdout was accepted (SPEC 7)"
  exit 1
fi
echo "  ✓ refused"

echo "▪ ledger"
./target/debug/sov ledger verify .demo/runs.jsonl
