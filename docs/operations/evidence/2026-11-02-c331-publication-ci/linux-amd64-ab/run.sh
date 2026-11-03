#!/bin/sh
set -x
uname -m; rustc -vV | head -2
for tree in head base; do
  cd /$tree
  CARGO_TARGET_DIR=/targets/$tree CARGO_INCREMENTAL=0 CARGO_NET_OFFLINE=true \
    cargo build --offline --locked --release -p benchmark-runner 2>&1 | tail -5
  echo "build $tree exit=$?"
done
echo "=== HEAD (a33e96d8; engine-core == 96236c96) ==="
for i in 1 2 3; do /targets/head/release/benchmark-runner start-hash; done
echo "=== BASELINE (9f4b7423) ==="
for i in 1 2 3; do /targets/base/release/benchmark-runner start-hash; done
