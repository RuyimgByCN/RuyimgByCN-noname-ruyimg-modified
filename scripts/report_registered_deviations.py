#!/usr/bin/env python3
"""统计台账里「已登记差异」的行数。

口径：对每个台账行，在该行为的**行为臂**（`filtering`/`metadata`/`step` 全部文件）
里查找 `差 异`/`差異` 关键字；命中即算「已登记」。

用法：
    python3 -B scripts/report_registered_deviations.py
"""

from __future__ import annotations

import sys
from pathlib import Path

import report_phase_arms as phase

HOST = Path(__file__).resolve().parent.parent / "crates/engine-native/src/content/host"
LEDGER = (
    Path(__file__).resolve().parent.parent
    / "docs/operations/evidence/2026-10-19-defect-ledger/ledger.tsv"
)
MARKERS = ("差异", "差異", "登記差異", "登记差异")


def arm_text(behavior: str, cached: dict[Path, str] | None = None) -> str:
    """返回该行为的全部行为臂文本。"""
    files = cached if cached is not None else {
        path: path.read_text(encoding="utf-8") for path in HOST.rglob("*.rs")
    }
    needle = f"HostBehavior::{behavior} =>"
    parts: list[str] = []
    for text in files.values():
        start = text.find(needle)
        while start >= 0:
            end = text.find("HostBehavior::", start + 10)
            parts.append(text[start : end if end > 0 else start + 900])
            start = text.find(needle, start + 10)
    return "\n".join(parts)


def registered(cached: dict[Path, str] | None = None) -> tuple[list[str], list[str]]:
    """返回（已登记差异的行为, 未登记差异的行为）。"""
    files = cached if cached is not None else {
        path: path.read_text(encoding="utf-8") for path in HOST.rglob("*.rs")
    }
    rows = phase.ledger_rows() if hasattr(phase, "ledger_rows") else _rows()
    yes: list[str] = []
    no: list[str] = []
    for behavior in rows:
        text = arm_text(behavior, files)
        (yes if any(marker in text for marker in MARKERS) else no).append(behavior)
    return yes, no


def _rows() -> list[str]:
    """读台账里的行为名。"""
    if not LEDGER.exists():
        return []
    lines = LEDGER.read_text(encoding="utf-8").splitlines()[1:]
    return [line.split("\t")[2] for line in lines if line.strip()]


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    yes, no = registered()
    print(f"台账 {len(yes) + len(no)} 行；已登记差异 {len(yes)} 行；未登记 {len(no)} 行")
    if "--check" in args:
        # 门禁口径：总数必须为 54，已登记数必须为 20（否则文档口径需同步更新）
        if len(yes) + len(no) != 6 or len(yes) != 2:
            print(f"❌ 口径漂移：总 {len(yes) + len(no)}（应 6）、已登记 {len(yes)}（应 2）")
            return 1
        print("✅ 口径一致：6 行 / 已登记 2 行")
        return 0
    for behavior in yes:
        print(f"  [差异] {behavior}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
