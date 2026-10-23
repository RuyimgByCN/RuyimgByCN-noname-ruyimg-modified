#!/usr/bin/env python3
"""门禁：禁止在宿主过滤臂/步骤里用 `card_kind` 做**相等比较**来判断「同一张牌」。

为什么（第 1001 轮的实际缺陷）：
上游 `player.countCards("he", card.name)` 比的是**牌名**；原生曾写成
`ctx.catalog.card_kind(*o) == k`——而**所有装备牌**的 `kind` 都是 `CardKind::Weapon/Armor/...`
（或统一装备类），于是「手里有任意一把武器」就让条件恒假，比上游严格得多。

判定规则：
* 扫描 `crates/engine-native/src/content/host/{filtering,step}/*.rs`；
* 命中 `card_kind(...) == ctx.catalog.card_kind(...)` / `!=` / `any(|o| ctx.catalog.card_kind` 形态；
* `ALLOWED` 里登记已人工核对过的例外（附理由）。

退出码：0 表示无新增；1 表示发现未登记的比较。
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET_DIRS = [
    ROOT / "crates/engine-native/src/content/host/filtering",
    ROOT / "crates/engine-native/src/content/host/step",
]

# 已人工核对：与上游语义一致，或比较对象不是「同一张牌」
ALLOWED = {
    # 判定区/装备区「可移动目标」比较，上游用 `card.name` 之外的槽位/种类判断（第 1002 轮核对）
    "XianjianLiguang",
}

PATTERNS = [
    re.compile(r"card_kind\([^)]*\)\s*==\s*ctx\.catalog\.card_kind"),
    re.compile(r"card_kind\([^)]*\)\s*!=\s*ctx\.catalog\.card_kind"),
    re.compile(r"any\(\|o\|\s*ctx\.catalog\.card_kind"),
]


def scan() -> list[tuple[str, str, int]]:
    out: list[tuple[str, str, int]] = []
    for d in TARGET_DIRS:
        for f in sorted(d.glob("*.rs")):
            text = f.read_text(encoding="utf-8")
            lines = text.splitlines()
            # 找出每个 HostBehavior 块的范围
            marks = [(m.start(), m.group(1)) for m in re.finditer(r"HostBehavior::(\w+)\s*=>", text)]
            for i, (start, beh) in enumerate(marks):
                end = marks[i + 1][0] if i + 1 < len(marks) else len(text)
                block = text[start:end]
                for pat in PATTERNS:
                    for m in pat.finditer(block):
                        line_no = text[: start + m.start()].count("\n") + 1
                        out.append((beh, lines[line_no - 1].strip(), line_no))
    return out


def main() -> int:
    hits = scan()
    bad = [h for h in hits if h[0] not in ALLOWED]
    print(f"扫描 {len(TARGET_DIRS)} 个目录，命中 {len(hits)} 处 card_kind 相等比较")
    for beh, snippet, line in hits:
        flag = "OK " if beh in ALLOWED else "❌ "
        print(f"  {flag}{beh}:{line}  {snippet[:90]}")
    if bad:
        print(f"\n❌ 发现 {len(bad)} 处未登记的 card_kind 相等比较——上游多半比的是 `card.name`，请改用牌名或登记理由")
        return 1
    print("\n✅ 所有 card_kind 相等比较均已登记")
    return 0


if __name__ == "__main__":
    sys.exit(main())
