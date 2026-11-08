#!/usr/bin/env python3
"""检查 `docs/**/*.md` 里的相对链接是否存在（排除 http/锚点）。

**为什么需要**：相对链接**写错就不生效，且没人会立刻发现** ——
本会话因此修过 5 条坏链（`../../` 少一层）。把它做成脚本 + 单测，
就能在 `unittest discover -s scripts` 里自动跑。

用法：
    python3 -B scripts/check_local_links.py            # 只检查有入链的主稿
    python3 -B scripts/check_local_links.py --all      # 检查 docs/ 下全部 md
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
LINK = re.compile(r"\]\(([^)\s]+)\)")

# 历史证据里「目标确实不存在」的已知坏链（**已登记，不是遗漏**）：
# - docs/adr/ 3 条：引用的 ADR 文件从未创建
# - 其余 4 条：引用的是「当时存在、现在已移动/未入库」的路径
KNOWN_BAD = {
    "0721-native-runner-baseline.md",
    "0730-native-n10-view-isolation.md",
    "0731-native-n10-choice-events.md",
    "../../archive/",
    "../../../crates/engine-native/tests/g1_cardpile.rs",
    "../native-coverage-manifest.md",
    "../2026-10-11-t15-abi-reconcile/README.md",
    "../../../../../ABI-COORDINATION.md",
}

# 2026-10-31 未跟踪历史报告（按「本地排除物全保留」原样留存）引用的 /tmp 与 ~/.pi 会话路径，
# 对应产物已归档入 docs/operations/evidence/2026-10-31-*，外部临时副本由系统清理：
KNOWN_BAD_PREFIXES = (
    "/tmp/i02-phase1-ingress-repair-89364384/",
    "/tmp/nc10-own-36992565/",
    "/Users/ruyimg/.pi/agent/sessions/",
)

# 默认只查这几份（本会话维护的主稿与证据）；--all 才扫全 docs/
CORE = [
    "docs/architecture/native-progress.md",
    "docs/architecture/native-rules-baseline.md",
    "docs/migration/upstream-extension-reconstruction.md",
    "docs/operations/evidence/2026-10-19-session-handover/README.md",
    "docs/operations/evidence/2026-10-19-rules-diff-handover/README.md",
    "docs/operations/evidence/2026-10-19-source-signoff-32/README.md",
    "docs/operations/evidence/2026-10-19-phase-audit/README.md",
]


def check(path: Path) -> list[str]:
    """返回该文件里的坏链（相对路径不存在）。"""
    text = path.read_text(encoding="utf-8", errors="ignore")
    bad: list[str] = []
    for m in LINK.finditer(text):
        target = m.group(1)
        if target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        target = target.split("#", 1)[0]
        if not target:
            continue
        resolved = (path.parent / target).resolve()
        if (
            not resolved.exists()
            and target not in KNOWN_BAD
            and not target.startswith(KNOWN_BAD_PREFIXES)
        ):
            bad.append(target)
    return bad


def main(argv: list[str]) -> int:
    files = sorted(DOCS.rglob("*.md")) if "--all" in argv else [ROOT / p for p in CORE]
    total = broken = 0
    for f in files:
        if not f.exists():
            continue
        total += 1
        for bad in check(f):
            broken += 1
            print(f"坏链: {f.relative_to(ROOT)} -> {bad}")
    print(f"检查 {total} 个文件，坏链 {broken}")
    return 1 if broken else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
