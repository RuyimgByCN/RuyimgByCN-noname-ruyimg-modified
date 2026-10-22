#!/usr/bin/env python3
"""台账账目核对：每一次「移出」都必须在某个注册表里留痕。

**为什么需要**：第 912 轮用手算核对 54 → 40 时漏掉 `SwdMufengHeld` 的**效果缺陷**（它随触发面
等价判定一起被移出，却没有任何记录）；第 950 轮又发现 **11 行**在 G23 判据修正时同样「无声消失」。
⇒ 手算会漏，机器不会。

核对方式：取基线版本（默认 `HEAD`）的台账行为集合，与当前台账对比，
凡「基线有、当前没有」的行为，必须能在下列任一注册表里找到：
`FIXED` / `NON_DEFECT_VERIFIED` / `NON_DEFECT_BY_CODE` / `REGISTERED_DIFFERENCE` / `UNRESOLVED_ATTRIBUTION`。

用法：
    python3 -B scripts/report_ledger_accounting.py [--rev <git-ref>]
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEDGER = "docs/operations/evidence/2026-10-19-defect-ledger/ledger.tsv"


def behaviors(text: str) -> set[str]:
    """台账文本 → 行为名集合（跳过表头与空行）。"""
    out: set[str] = set()
    for line in text.splitlines()[1:]:
        if not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) >= 3:
            out.add(parts[2])
    return out


def git_ledger(rev: str, root: Path = ROOT) -> str:
    result = subprocess.run(
        ["git", "show", f"{rev}:{LEDGER}"], cwd=root, capture_output=True, text=True
    )
    return result.stdout if result.returncode == 0 else ""


def unaccounted(
    removed: set[str], registries: list[dict[str, str]]
) -> list[str]:
    """移出但不在任何注册表里的行为。"""
    return sorted(b for b in removed if not any(b in reg for reg in registries))


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    rev = "HEAD"
    if "--rev" in args:
        rev = args[args.index("--rev") + 1]
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import report_defect_ledger as ledger  # 延迟导入：避免 preflight 启动开销

    base = behaviors(git_ledger(rev))
    current = behaviors((ROOT / LEDGER).read_text(encoding="utf-8"))
    if not base:
        print(f"⚠ 取不到 {rev} 的台账（跳过核对）")
        return 0
    removed = base - current
    added = current - base
    bad = unaccounted(
        removed,
        [
            ledger.FIXED,
            ledger.NON_DEFECT_VERIFIED,
            ledger.NON_DEFECT_BY_CODE,
            ledger.REGISTERED_DIFFERENCE,
            ledger.UNRESOLVED_ATTRIBUTION,
        ],
    )
    print(f"{rev}: {len(base)} 个行为 → 当前 {len(current)}（移出 {len(removed)}、新增 {len(added)}）")
    if bad:
        print(f"❌ {len(bad)} 行移出但未登记：{' '.join(bad)}")
        return 1
    print("✅ 移出的每一行都能在注册表里找到")
    return 0


if __name__ == "__main__":
    sys.exit(main())
