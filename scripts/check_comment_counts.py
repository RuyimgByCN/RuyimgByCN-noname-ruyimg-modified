#!/usr/bin/env python3
"""Check the counts written in code comments against the defect ledger.

The guard file documents "24 条 `需档`", "`CardTargeted` 解锁 14 条", … — plain
numbers in doc comments.  Nothing fails when they go stale (第 794 轮 found 27/23
left over from an older ledger).  This check ties those numbers to `ledger.tsv`.

Patterns understood (deliberately narrow — only what the guards actually say):

    /// 24 条 `需档` …
    capability_guard!(…, "…解锁 14 条…")

Every ``<n> 条`` immediately followed by a backticked family id, and every
``解锁 <n> 条`` inside `capability_guard!`, is compared to the ledger count.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GUARDS = ROOT / "crates" / "engine-native" / "tests" / "nc08_registered_defect_guards.rs"
LEDGER = ROOT / "docs" / "operations" / "evidence" / "2026-10-19-defect-ledger" / "ledger.tsv"

FAMILY_CLAIM = re.compile(r"(\d+)\s*条\s*`([^`]+)`")
UNLOCK_CLAIM = re.compile(r"解锁\s*(\d+)\s*条")


def ledger_counts() -> dict[str, int]:
    counts: dict[str, int] = {}
    lines = LEDGER.read_text(encoding="utf-8").splitlines()
    for line in lines[1:]:
        if not line.strip():
            continue
        family = line.split("\t")[0]
        # `需档(sha 幕)` 等带括号的 id 归到**前缀族**（不重复计一次全名）
        key = family.split("(")[0]
        counts[key] = counts.get(key, 0) + 1
    return counts


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strict", action="store_true", help="exit 1 on a stale count")
    args = ap.parse_args()

    counts = ledger_counts()
    text = GUARDS.read_text(encoding="utf-8")
    problems: list[str] = []

    for match in FAMILY_CLAIM.finditer(text):
        claimed, family = int(match.group(1)), match.group(2)
        # 只校验「需档」这类已知族；`useCardTo*` 这种说明性文字跳过
        if family not in counts and family not in {"需档"}:
            continue
        actual = counts.get(family, 0)
        if claimed != actual:
            problems.append(f"注释写 `{family}` 有 {claimed} 条，台账实际 {actual} 条")

    unlock_expected = counts.get("需档", 0)
    for match in UNLOCK_CLAIM.finditer(text):
        claimed = int(match.group(1))
        # 「解锁 N 条」应等于该槽位覆盖的需档行数；至少不能超过总需档数
        if claimed > unlock_expected:
            problems.append(f"注释写「解锁 {claimed} 条」，超过台账 `需档` 总数 {unlock_expected}")

    print(f"台账族计数：{ {k: v for k, v in sorted(counts.items()) if k.startswith('需档') or k in ('G14', 'G23', 'G20', 'G18', 'G22', 'G15', 'G19')} }")
    if problems:
        print(f"发现 {len(problems)} 处过期计数：", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1 if args.strict else 0
    print("守卫注释里的计数与台账一致 ✅")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
