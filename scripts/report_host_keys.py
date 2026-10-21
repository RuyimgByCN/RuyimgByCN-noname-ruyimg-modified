#!/usr/bin/env python3
"""Count `host:` keys per source package — the wiring census behind the rules baseline.

A `host:` key is what a package declares to bind a skill/card behaviour to the
native host (`host:<key>`).  The key can appear in **any** field of the asset JSON
(`skills[].mapping`, `cards[].equip_skills`, …), so the whole document text is
scanned — counting only `skills[].mapping` undercounts by ~9× (U05: 27 vs 239,
see `docs/architecture/native-rules-baseline.md` 第 761 轮).

Usage:
    python3 -B scripts/report_host_keys.py
    python3 -B scripts/report_host_keys.py --check
"""

from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKAGES = ROOT / "packages" / "extensions"

#: 2026-10-19 第 776 轮实测（U05 家族的 8 个子包）；口径变化须同步更新规则基准。
#: U05 由 **9 个包**组成（第 776 轮更正：此前只算了 8 个，漏了 `shaai-xianjian` 与 `shahai-zhenfa`）。
EXPECTED_U05 = {
    "shahai-swd-upstream-noname": 187,
    "shaai-xianjian-upstream-noname": 75,
    "shaai-ow-upstream-noname": 36,
    "shahai-yunchou-upstream-noname": 16,
    "shahai-gujian-upstream-noname": 0,
    "shahai-hearth-upstream-noname": 0,
    "shahai-gwent-upstream-noname": 0,
    "shahai-mtg-upstream-noname": 0,
    "shahai-zhenfa-upstream-noname": 0,
}

KEY_RE = re.compile(r"host:[a-z0-9_]+")


def keys_for(package: str) -> set[str]:
    found: set[str] = set()
    for asset in glob.glob(str(PACKAGES / package / "assets" / "*.json")):
        try:
            doc = json.loads(Path(asset).read_text(encoding="utf-8"))
        except Exception:
            continue
        found |= set(KEY_RE.findall(json.dumps(doc, ensure_ascii=False)))
    return found


def survey() -> dict[str, int]:
    out: dict[str, int] = {}
    for path in sorted(PACKAGES.iterdir()):
        if not path.is_dir():
            continue
        count = len(keys_for(path.name))
        if count:
            out[path.name] = count
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when U05 counts drift from EXPECTED_U05")
    args = ap.parse_args()

    per_package = survey()
    u05 = {name: per_package.get(name, 0) for name in EXPECTED_U05}
    u05_total = 0
    for name, expected in EXPECTED_U05.items():
        actual = u05[name]
        u05_total += actual
        mark = "" if actual == expected else f"  ⚠ 期望 {expected}"
        print(f"{name:34s} {actual:4d}{mark}")
    print(f"{'U05 合计（9 包）':32s} {u05_total:4d}")

    others = {name: count for name, count in per_package.items() if name not in EXPECTED_U05}
    if others:
        print("\n其余含 host 键的包：")
        for name, count in sorted(others.items(), key=lambda kv: -kv[1]):
            print(f"  {name:32s} {count:4d}")
    print(f"\n全部包合计唯一键数（按包求和）: {sum(per_package.values())}", file=sys.stderr)

    drift = {name: (exp, u05[name]) for name, exp in EXPECTED_U05.items() if exp != u05[name]}
    if drift:
        for name, (expected, actual) in drift.items():
            print(f"⚠ {name}: 期望 {expected}，实测 {actual}", file=sys.stderr)
        if args.check:
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
