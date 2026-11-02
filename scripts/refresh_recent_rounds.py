#!/usr/bin/env python3
"""刷新 `native-progress.md` 头部的「最近 N 轮索引」块。

**为什么需要**：进度主稿按轮次**只追加**，最新内容永远在最深处；
头部索引必须跟着更新，否则**索引本身会腐化**（本会话已修过 3 次「头部腐化」）。

用法：
    python3 -B scripts/refresh_recent_rounds.py          # 重写索引块
    python3 -B scripts/refresh_recent_rounds.py --check   # 只检查是否过期（exit 1 表示过期）
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# 2026-11-03 起指向历史稿：轮次正文随裁决 32② 从主稿逐字节移存到
# native-progress-history.md；主稿自 2026-10-31 起改用带日期条目，不再按轮次追加。
DOC = ROOT / "docs/architecture/native-progress-history.md"
N = 20
BEGIN = "> ### **最近 20 轮索引**"
END = "> **更早的内容**"


def rounds(text: str) -> list[tuple[int, str]]:
    found = {
        (int(m.group(1)), m.group(2).strip())
        for m in re.finditer(r"> \*\*第 (\d+) 轮\*\*：([^\n]{0,80})", text)
    }
    return sorted(found)[-N:]


def render(items: list[tuple[int, str]]) -> str:
    body = "\n".join(f"> - **第 {n} 轮**：{t[:52]}…" for n, t in items)
    return (
        f"{BEGIN}（最新在底部；正文按轮次追加）\n>\n{body}\n>\n"
        f"{END}**：按轮次向下检索（`第 N 轮`），或直接读"
        f"[交接索引](../operations/evidence/2026-10-19-session-handover/README.md)。"
    )


def main(argv: list[str]) -> int:
    text = DOC.read_text(encoding="utf-8")
    items = rounds(text)
    fresh = render(items)

    start = text.find(BEGIN)
    end = text.find(END)
    if start < 0 or end < 0:
        print("找不到索引块标记", file=sys.stderr)
        return 2
    # 块结尾 = END 那一行的行尾
    end = text.find("\n", end)
    current = text[start:end]

    if "--check" in argv:
        stale = current.strip() != fresh.strip()
        print("索引" + ("已过期" if stale else "是最新") + f"（{len(items)} 轮）")
        return 1 if stale else 0

    if current.strip() == fresh.strip():
        print(f"索引已是最新（{len(items)} 轮）")
        return 0
    DOC.write_text(text[:start] + fresh + text[end:], encoding="utf-8")
    print(f"索引已刷新（{len(items)} 轮）")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
