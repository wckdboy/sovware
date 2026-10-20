#!/usr/bin/env bash
# The loop, end to end, offline. This is the test in the manifesto (§13).
set -uo pipefail
cd "$(dirname "$0")/.."

SOV=${SOV:-./target/debug/sov}
LEDGER=.demo/runs.jsonl
rm -rf .demo && mkdir -p .demo

fail=0
RUN_ID=""

# verify_once <label> <expected-exit> <program> <actor>; sets RUN_ID
verify_once() {
  local label=$1 expected=$2 program=$3 actor=$4
  "$SOV" verify "$program" --ledger "$LEDGER" --actor "$actor" > ".demo/$label.out" 2>&1
  local code=$?
  cat ".demo/$label.out"
  if [ "$code" -ne "$expected" ]; then
    echo "✗ $label: expected exit $expected, got $code"
    fail=1
  fi
  echo
  RUN_ID=$(tail -1 "$LEDGER" | grep -o '"id":"run_[a-f0-9]*"' | head -1 | cut -d'"' -f4)
}

echo "▪ sovware loop"
echo

verify_once v1 1 examples/bracket/lug-bracket.v1.json human:wd
v1=$RUN_ID
verify_once v2 0 examples/bracket/lug-bracket.v2.json agent:nightfall
v2=$RUN_ID
verify_once v3 1 examples/bracket/lug-bracket.v3.json agent:nightfall
v3=$RUN_ID

echo "▪ improvement v1 → v2"
"$SOV" compare "$v1" "$v2" --ledger "$LEDGER" | tee .demo/compare-12.out
echo

echo "▪ the cheaper mutation v1 → v3"
"$SOV" compare "$v1" "$v3" --ledger "$LEDGER" | tee .demo/compare-13.out
echo

grep -q "improvement" .demo/compare-12.out || { echo "✗ v1 → v2 must be an improvement"; fail=1; }
grep -q "change" .demo/compare-13.out || { echo "✗ v1 → v3 must not be an improvement"; fail=1; }

echo "▪ ledger"
"$SOV" ledger verify "$LEDGER" || fail=1

exit "$fail"
