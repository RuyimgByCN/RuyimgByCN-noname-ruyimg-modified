#!/usr/bin/env python3
"""Report which ledger defects have an executable `#[ignore]` acceptance guard.

Every defect in `ledger.tsv` should be backed by a guard in
`crates/engine-native/tests/nc08_registered_defect_guards.rs` (or the older
`nc08_upstream_*.rs` files) so that "fixed" can be proved by removing the
`#[ignore]` and watching it go green.

This script measures the gap: it extracts the `HostBehavior` names mentioned in
the guard test files and matches them against the ledger behaviours.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TESTS = ROOT / "crates" / "engine-native" / "tests"
LEDGER = ROOT / "docs" / "operations" / "evidence" / "2026-10-19-defect-ledger" / "ledger.tsv"

#: Ledger ids whose guard is a source-fact assertion on the *filter* rather than a
#: behaviour name; the guard text still mentions the behaviour, so plain matching works.
BEH_RE = re.compile(r"HostBehavior::([A-Za-z0-9_]+)")
GUARD_RE = re.compile(r"HostBehavior::([A-Za-z0-9_]+)")
#: 宏生成的守卫把变体名当**字符串字面量**传参（`g14_guard!(…, "SwdHuanxing", …)`），
#: 所以还要收集「双引号里的 CamelCase 标识符」。
QUOTED_RE = re.compile(r'"([A-Z][A-Za-z0-9_]*)"')


def guard_behaviors() -> set[str]:
    out: set[str] = set()
    for path in TESTS.glob("nc08_*.rs"):
        try:
            text = path.read_text(encoding="utf-8")
        except Exception:
            continue
        if "#[ignore" not in text:
            continue
        out |= set(GUARD_RE.findall(text))
        out |= set(QUOTED_RE.findall(text))
    return out


#: 能力守卫解锁的缺陷族：`cap_*` 守卫断言枚举槽位，按「它覆盖哪些 id」计入。
CAPABILITY_IDS = {
    "cap_card_targeted": {"需档"},
    "cap_sha_begin": {"需档"},
    "cap_sha_after": {"需档"},
    "cap_sha_before": {"需档"},
    "cap_death_before": {"需档"},
    "cap_game_start": {"L20b"},
    "cap_judge_before": {"L03c"},
    "cap_damage_begin_second": {"第3次"},
    "guard_prepare_phase_exists": {"G18"},
    "guard_xiangxing_lose_hp_end": {"L03a"},
    "guard_yueyin_fourth_damage": {"L03b"},
    "guard_gaiming_judge_before": {"L03c"},
    "guard_kuwu_and_shengke_card_targeted": {"5a", "5c"},
    "guard_mopaifanbei_second_draw": {"L20a"},
    "guard_rangjie_branch1_draws": {"6"},
}


def capability_guard_names() -> set[str]:
    text = (TESTS / "nc08_registered_defect_guards.rs").read_text(encoding="utf-8")
    # 守卫可能由宏生成：源码里是 `capability_guard!(cap_x, ...)` 而不是 `fn cap_x(`
    return {
        name
        for name in CAPABILITY_IDS
        if f"fn {name}(" in text or f"capability_guard!({name}" in text
    }


def ledger_rows() -> list[list[str]]:
    lines = LEDGER.read_text(encoding="utf-8").splitlines()
    return [line.split("\t") for line in lines[1:] if line.strip()]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when coverage is below --min")
    ap.add_argument("--min", type=float, default=0.0, help="minimum coverage ratio (0..1)")
    args = ap.parse_args()

    guarded = guard_behaviors()
    caps = capability_guard_names()
    capability_ids: set[str] = set()
    for name in caps:
        capability_ids |= CAPABILITY_IDS[name]
    rows = ledger_rows()
    covered = []
    missing = []
    for row in rows:
        behavior = row[2]
        # 守卫文本里可能写的是变体名（`SwdHuanxia`）也可能只写行为片段
        if row[0] in capability_ids or behavior in guarded:
            covered.append(behavior)
        else:
            missing.append((row[0], behavior))

    ratio = len(covered) / len(rows) if rows else 0.0
    print(f"台账 {len(rows)} 条；有守卫 {len(covered)} 条；缺守卫 {len(missing)} 条；覆盖率 {ratio:.1%}")
    if missing:
        print("\n缺守卫的缺陷（按 id 分组）：", file=sys.stderr)
        by_id: dict[str, list[str]] = {}
        for rid, behavior in missing:
            by_id.setdefault(rid, []).append(behavior)
        for rid, items in sorted(by_id.items(), key=lambda kv: -len(kv[1])):
            print(f"  {rid:8s} {len(items):2d}  {', '.join(items[:6])}{' …' if len(items) > 6 else ''}", file=sys.stderr)

    if args.check and ratio < args.min:
        print(f"\n覆盖率 {ratio:.1%} < 门槛 {args.min:.1%}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
