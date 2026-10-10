#!/usr/bin/env bash
# Cross-target golden hash gate (Phase 13).
# Conformance pins the golden vectors; this script additionally proves that the
# native and browser WASM runtimes produce the same hashes from current source.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "== native conformance =="
cargo run -q -p conformance

echo "== native start hashes =="
H42=$(cargo run -q -p engine-cli -- hash 4 42)
H1=$(cargo run -q -p engine-cli -- hash 2 1)
echo "start_4_42=$H42"
echo "start_2_1=$H1"

echo "== native manifest hashes (Phase 14) =="
M42=$(cargo run -q -p engine-cli -- manifest 4 42 | awk -F= '/^manifest_hash/{print $2}')
echo "manifest_4_42=$M42"
test -n "$M42"
cargo run -q -p engine-cli -- manifest 4 42 | grep -q 'chain=ok'

echo "== browser wasm runtime hashes =="
rustup target add wasm32-unknown-unknown >/dev/null
pnpm --dir apps/web-client build:wasm
W42=$(node scripts/wasm-hash.mjs 4 42)
W1=$(node scripts/wasm-hash.mjs 2 1)
echo "wasm_start_4_42=$W42"
echo "wasm_start_2_1=$W1"
test "$H42" = "$W42"
test "$H1" = "$W1"

echo "== content package skill-vm smoke =="
cargo run -q -p engine-cli -- skill-vm content/standard/jianxiong.skill | grep -q 'ok=true'

echo "ALL CROSS-TARGET CHECKS PASSED"
