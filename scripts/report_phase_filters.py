#!/usr/bin/env python3
"""重新生成「阶段过滤审计登记表」（52 条）的骨架，**保留已填的状态列**。

**为什么需要**：`docs/operations/evidence/2026-10-19-phase-audit/registry.txt`
最初由一段临时脚本生成，**没有入库** ⇒ 无法复现、无法在 filter 改动后重新核对。
本脚本让它可以「随时重生成」，并**保留人工填好的 `状态`/`依据` 两列**。

**切片规则见 `arms()`**（段切片，不截断到第一个 `=>`）——
切片方式会改变条数（52 vs 51），详见
`docs/operations/evidence/2026-10-19-g14-guard/slicing.md`。

用法：
    python3 -B scripts/report_phase_filters.py            # 重生成 registry.txt
    python3 -B scripts/report_phase_filters.py --check     # 只比对，不一致则 exit 1
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates/engine-native/src/content/host"
KEYS = HOST.parents[1] / "vocabulary/host/keys.rs"
REG = ROOT / "docs/operations/evidence/2026-10-19-phase-audit/registry.txt"

PHASE = re.compile(r"phase:\s*crate::event::Phase::(\w+),\s*begin:\s*(true|false)")
BEHAVIOR = re.compile(r"HostBehavior::(\w+)\s*=>")
KEY = re.compile(r'HostBehavior::(\w+)\s*=>\s*"host:([^"]+)"')


def arms() -> list[tuple[str, str]]:
    """按「段切片」返回 [(行为, 臂体)]。

    切片规则：从 `HostBehavior::X =>` 到**下一个** `HostBehavior::`（**不截断到第一个 `=>`**）。
    理由见 docs/operations/evidence/2026-10-19-g14-guard/slicing.md —— 截断会丢掉
    `Xiefang` 那种跨行 `||` 的多分支臂。
    """
    files = sorted(HOST.glob("filtering*.rs")) + sorted(HOST.glob("filtering/*.rs"))
    out: list[tuple[str, str]] = []
    for f in files:
        text = f.read_text(encoding="utf-8")
        marks = [(m.start(), m.group(1)) for m in BEHAVIOR.finditer(text)]
        for i, (pos, beh) in enumerate(marks):
            end = marks[i + 1][0] if i + 1 < len(marks) else len(text)
            out.append((beh, text[pos:end]))
    return out


def scan() -> list[tuple[str, str, str, str]]:
    """返回 [(行为, key, Phase, begin)]，按文件顺序。"""
    keys = {m.group(1): m.group(2) for m in KEY.finditer(KEYS.read_text(encoding="utf-8"))}
    keys = {m.group(1): m.group(2) for m in KEY.finditer(KEYS.read_text(encoding="utf-8"))}
    rows: list[tuple[str, str, str, str]] = []
    for beh, body in arms():
        for pm in PHASE.finditer(body):
            rows.append((beh, keys.get(beh, ""), pm.group(1), pm.group(2)))
    return rows


def previous_status() -> dict[str, tuple[str, str]]:
    """从现有 registry 里读回 (状态, 依据)，按行为名索引。"""
    out: dict[str, tuple[str, str]] = {}
    if not REG.exists():
        return out
    for line in REG.read_text(encoding="utf-8").splitlines()[1:]:
        cells = line.split("\t")
        if len(cells) >= 6:
            out[cells[0]] = (cells[4], cells[5])
    return out


def render(rows: list[tuple[str, str, str, str]]) -> str:
    prev = previous_status()
    lines = ["行为\tkey\tPhase\tbegin\t状态\t依据"]
    for beh, key, phase, begin in rows:
        status, why = prev.get(beh, ("未核", ""))
        lines.append(f"{beh}\t{key}\t{phase}\t{begin}\t{status}\t{why}")
    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    rows = scan()
    fresh = render(rows)
    old = REG.read_text(encoding="utf-8") if REG.exists() else ""
    if "--check" in argv:
        same = old.strip() == fresh.strip()
        print(f"registry {'一致' if same else '不一致'}（{len(rows)} 条）")
        return 0 if same else 1
    if old.strip() == fresh.strip():
        print(f"registry 已是最新（{len(rows)} 条）")
        return 0
    REG.write_text(fresh, encoding="utf-8")
    print(f"registry 已重生成（{len(rows)} 条，状态列已保留）")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
