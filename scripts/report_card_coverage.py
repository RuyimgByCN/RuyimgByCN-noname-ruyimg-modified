#!/usr/bin/env python3
"""Card-face coverage: are the extension packs' cards registered *and* wired?

The card JSON only carries identity (`name/kind/suit/rank`) — no `effect` field.
Effects dispatch on `kind`, so coverage has three layers:

1. **registered** — `crates/engine-native-kernel/src/catalog.rs` maps `CardKind::X => "kind"`
2. **wired** — the kind (as `CardKind::X` variant *or* bare `kind` literal) appears in
   some other `crates/**/*.rs`
3. **behaviour correct** — NOT measurable here; needs a per-card read of the effect step

Layer 2 is deliberately broad: an earlier version scanned only `content/host/` and
required the `CardKind::X` form, which produced a wrong 48/128 (see
`docs/architecture/native-rules-baseline.md`, 第 774 轮).  Keep the scope wide.

Usage:
    python3 -B scripts/report_card_coverage.py
    python3 -B scripts/report_card_coverage.py --check
"""

from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crates" / "engine-native-kernel" / "src" / "catalog.rs"
# catalog 模块的子文件（`impl CardKind` 等已拆到 catalog/ 下）一并视为目录本身。
CATALOG_DIR = ROOT / "crates" / "engine-native-kernel" / "src" / "catalog"


def catalog_files() -> list[Path]:
    return [CATALOG] + (sorted(CATALOG_DIR.rglob("*.rs")) if CATALOG_DIR.is_dir() else [])

#: 六个带卡牌的 U05 子包（xianjian / ow 的 cards 为空）。
PACKS = [
    "shahai-swd-upstream-noname",
    "shahai-gujian-upstream-noname",
    "shahai-yunchou-upstream-noname",
    "shahai-hearth-upstream-noname",
    "shahai-gwent-upstream-noname",
    "shahai-mtg-upstream-noname",
]

#: 2026-11-08 裁决 33 实测值（cangchizhibi 入库后 201→202 张、128→129 kind）；口径变化时必须同步更新这里与规则基准的说明。
EXPECTED = {"cards": 202, "kinds": 129, "registered": 129, "wired": 129}


def pack_cards() -> dict[str, int]:
    out: dict[str, int] = {}
    for pack in PACKS:
        count = 0
        for asset in glob.glob(str(ROOT / "packages" / "extensions" / pack / "assets" / "*.json")):
            try:
                doc = json.loads(Path(asset).read_text(encoding="utf-8"))
            except Exception:
                continue
            for card in doc.get("cards") or []:
                if isinstance(card, dict) and card.get("kind"):
                    count += 1
        out[pack] = count
    return out


def kinds() -> set[str]:
    found: set[str] = set()
    for pack in PACKS:
        for asset in glob.glob(str(ROOT / "packages" / "extensions" / pack / "assets" / "*.json")):
            try:
                doc = json.loads(Path(asset).read_text(encoding="utf-8"))
            except Exception:
                continue
            for card in doc.get("cards") or []:
                if isinstance(card, dict) and card.get("kind"):
                    found.add(card["kind"])
    return found


def catalog_map() -> dict[str, str]:
    text = "\n".join(p.read_text(encoding="utf-8") for p in catalog_files())
    return {kind: variant for variant, kind in re.findall(r'CardKind::([A-Za-z0-9_]+)\s*=>\s*"([a-z0-9_]+)"', text)}


def wired_kinds(all_kinds: set[str], mapping: dict[str, str]) -> set[str]:
    blobs = [
        path.read_text(encoding="utf-8", errors="replace")
        for path in (ROOT / "crates").rglob("*.rs")
        if path.name != "catalog.rs" and CATALOG_DIR not in path.parents
    ]
    wired = set()
    for kind in all_kinds:
        variant = mapping.get(kind)
        if variant and any(f"CardKind::{variant}" in blob for blob in blobs):
            wired.add(kind)
        elif any(kind in blob for blob in blobs):
            wired.add(kind)
    return wired


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when counts drift from EXPECTED")
    args = ap.parse_args()

    per_pack = pack_cards()
    all_kinds = kinds()
    mapping = catalog_map()
    registered = {kind for kind in all_kinds if kind in mapping}
    wired = wired_kinds(all_kinds, mapping)

    print(f"{'子包':24s} {'卡数':>5s}")
    for pack, count in per_pack.items():
        print(f"{pack:24s} {count:5d}")
    print(f"{'合计':24s} {sum(per_pack.values()):5d}")
    print()
    print(f"唯一 kind            {len(all_kinds):4d}")
    print(f"catalog 已登记        {len(registered):4d}  ({len(registered) / len(all_kinds):.1%})")
    print(f"catalog 之外有源码引用  {len(wired):4d}  ({len(wired) / len(all_kinds):.1%})")
    print(f"完全无引用            {len(all_kinds - wired):4d}")
    print("\n⚠ 「有引用」≠「行为正确」；效果正确性需逐张读效果步（本脚本不判断）。", file=sys.stderr)

    actual = {
        "cards": sum(per_pack.values()),
        "kinds": len(all_kinds),
        "registered": len(registered),
        "wired": len(wired),
    }
    drift = {key: (EXPECTED[key], actual[key]) for key in EXPECTED if EXPECTED[key] != actual[key]}
    if drift:
        for key, (expected, got) in drift.items():
            print(f"⚠ {key}: 期望 {expected}，实测 {got}", file=sys.stderr)
        if args.check:
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
