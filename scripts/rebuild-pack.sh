#!/usr/bin/env bash
# Rebuild a ported content package npack from its .skill sources + roster + metadata + assets.
# Usage: scripts/rebuild-pack.sh <package_id>
# Rebuilds content/original/ported/<pkg>/<pkg>.npack.json preserving assets/logic.
set -euo pipefail
PKG="${1:?usage: rebuild-pack.sh <package_id>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
PDIR="content/original/ported/$PKG"
[[ -d "$PDIR" ]] || { echo "no such package dir $PDIR"; exit 1; }

# Concatenate all .skill files in deterministic (byte) order. LC_ALL=C makes
# `sort` byte-wise: locale collation (en_US.UTF-8) orders e.g. `oljiangchi_less`
# before `oljiangchi`, which disagrees with the compiler/tests' byte-order sort
# and silently changes every package logic_hash per environment.
SKILL_LIST=$(ls "$PDIR"/*.skill 2>/dev/null | LC_ALL=C sort)
[[ -n "$SKILL_LIST" ]] || { echo "no .skill files in $PDIR"; exit 1; }

# Build a combined metadata JSON that includes the existing asset_manifest + asset_paths
# (so assets are preserved exactly without recompiling the asset directory).
python3 - "$PKG" << 'PY'
import json, sys, os
pkg = sys.argv[1]
pdir = f"content/original/ported/{pkg}"
meta = json.load(open(f"{pdir}/metadata.json"))
npack_path = f"{pdir}/{pkg}.npack.json"
if os.path.exists(npack_path):
    npack = json.load(open(npack_path))
    if npack.get("asset_manifest") is not None:
        meta["asset_manifest"] = npack["asset_manifest"]
    if npack.get("asset_paths"):
        meta["asset_paths"] = npack["asset_paths"]
json.dump(meta, open(f"/tmp/rebuild_meta_{pkg}.json","w"), ensure_ascii=False, indent=2)
PY

# Collect .skill inputs
INPUTS=()
for f in $SKILL_LIST; do INPUTS+=("$f"); done

# Include --cards if a cards.json exists
CARDS_ARG=""
[[ -f "$PDIR/cards.json" ]] && CARDS_ARG="--cards $PDIR/cards.json"

# Include --characters only if characters.json exists (some packages have no characters)
CHARS_ARG=""
[[ -f "$PDIR/characters.json" ]] && CHARS_ARG="--characters $PDIR/characters.json"

cargo run -q -p content-cli -- pack \
  "${INPUTS[@]}" \
  --out "/tmp/rebuild_${PKG}.npack.json" \
  --package-id "$PKG" \
  --namespace official \
  --version 0.1.0 \
  ${CHARS_ARG:-} \
  ${CARDS_ARG:-} \
  --metadata "/tmp/rebuild_meta_${PKG}.json" \
  || { echo "PACK FAILED"; exit 1; }

echo "rebuilt /tmp/rebuild_${PKG}.npack.json"
