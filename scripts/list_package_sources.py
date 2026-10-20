#!/usr/bin/env python3
"""列出每个扩展包的「上游来源」，并统计「按来源」的包分布。

**为什么需要这个脚本**：包数据里**没有统一的来源字段** ——
- 有的包把 `upstream_path` 写在 `assets/config.json`；
- 有的包写在 `assets/cards.json`（如 `shaai-ow`/`shahai-swd` 等 shahai 子包）；
- 有的两处都有。
⇒ 任何「按来源统计」的脚本**必须同时读两处**，否则会得到互相矛盾的数字（本会话已多次踩坑）。

用法：
    python3 -B scripts/list_package_sources.py            # 表格
    python3 -B scripts/list_package_sources.py --summary   # 按来源汇总
"""

from __future__ import annotations

import json
import os
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PKGS = ROOT / "packages" / "extensions"


def upstream_path(pkg_dir: Path) -> str | None:
    """先看 config.json，再看 cards.json（两处都可能声明）。"""
    for name in ("config.json", "cards.json"):
        path = pkg_dir / "assets" / name
        if not path.is_file():
            continue
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            continue
        if isinstance(data, dict) and isinstance(data.get("upstream_path"), str):
            return data["upstream_path"]
    return None


def source_of(up: str) -> str:
    """把 upstream_path 归一成「来源目录名」。"""
    parts = up.split("extension/")
    if len(parts) > 1:
        return parts[1].split("/")[0]
    return up


def main(argv: list[str]) -> int:
    summary = "--summary" in argv
    rows: list[tuple[str, str, str]] = []
    for pkg in sorted(PKGS.iterdir()):
        if not pkg.is_dir():
            continue
        up = upstream_path(pkg)
        rows.append((pkg.name, up or "", source_of(up) if up else ""))

    if summary:
        by_src: dict[str, list[str]] = defaultdict(list)
        for name, _up, src in rows:
            by_src[src or "(未声明)"].append(name)
        for src in sorted(by_src):
            print(f"{src:18s} {len(by_src[src]):3d} 包：{', '.join(by_src[src])}")
        print(f"\n来源数（去重，含未声明）：{len(by_src)}；包数：{len(rows)}")
        return 0

    print(f"{'包':38s} {'来源':16s} upstream_path")
    for name, up, src in rows:
        print(f"{name:38s} {src or '(未声明)':16s} {up}")
    declared = sum(1 for _n, up, _s in rows if up)
    print(f"\n包数 {len(rows)}；有 upstream_path {declared}；未声明 {len(rows) - declared}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
