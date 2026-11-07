#!/usr/bin/env python3
"""把 `packages/extensions/*` 映射到台账的 37 个来源，输出「包 → 来源」表。

**为什么需要**：「八项按来源」的分母 32 = 37 − 5 个无包来源，
但这个映射**只能靠 `upstream_path` 的目录名 + 包名后缀**推出来，
而 `boss`/`cardpile`/`coin` **各有两个同名来源**（noname 与 liuli），
按名字匹配会把 liuli 的包记到 noname 上。本脚本把消歧规则固化下来。

消歧规则（按优先级）：
1. **包名后缀 `-noname` / `-liuli`** 决定仓库侧，再用 `upstream_path` 的目录名匹配；
2. `upstream_path` 为 `None` 的少数包，用 `EXPLICIT` 表（包名 → 来源 ID）；
3. 匹配不到 ⇒ 报「未映射」，**不猜**。

用法：
    python3 -B scripts/map_package_sources.py            # 打印汇总
    python3 -B scripts/map_package_sources.py --tsv F    # 写 TSV
    python3 -B scripts/map_package_sources.py --check    # 无未映射则 exit 0
"""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PACKAGES = ROOT / "packages/extensions"
LEDGER = ROOT / "docs/migration/upstream-extension-reconstruction.md"
ASSETS = ("config.json", "cards.json", "generals.json", "character.json")
ANCHORS = ("extension", "mode", "character", "card")

# 台账里没有可用的 upstream_path 的包（按包名后缀 + 台账目录名人工确认）
EXPLICIT = {
    "boss-zhugeliang-upstream-liuli": "L03",
    "cardpile-upstream-liuli": "L04",
    "cardpile-upstream-noname": "U03",
    "coin-upstream-noname": "U04",
    "desktop-adapt-upstream-liuli": "L23",
    "omnisearch-upstream-liuli": "L07",
}
REQUIRED = 37
NO_PACKAGE: set[str] = set()
# UP37 核心武将包（上游 `apps/core/character/*`，不属于两仓扩展台账的 37 个 extension/mode 来源）
CHARACTER_PACKAGES = {
    "sp2-upstream-noname",
    "xianding-upstream-noname",
}


def ledger_sources() -> dict[str, tuple[str, str, str]]:
    """返回 {来源ID: (名称, 仓库, 目录名)}，来源行形如 `| U01 [名](.../archive/upstream/<repo>/<path>)`。"""
    text = LEDGER.read_text(encoding="utf-8")
    out: dict[str, tuple[str, str, str]] = {}
    for m in re.finditer(r"\| ([UL]\d{2}) \[([^\]]+)\]\([^)]*archive/upstream/([^/]+)/(.*?)\)", text):
        sid, name, repo, path = m.group(1), m.group(2), m.group(3), m.group(4)
        parts = [p for p in path.split("/") if p]
        directory = ""
        for i, p in enumerate(parts):
            if p in ANCHORS and i + 1 < len(parts):
                directory = parts[i + 1]
                break
        out[sid] = (name, repo, directory)
    return out


def upstream_path(pkg: Path) -> str | None:
    for name in ASSETS:
        f = pkg / "assets" / name
        if not f.exists():
            continue
        try:
            data = json.loads(f.read_text(encoding="utf-8"))
        except Exception:
            continue
        if isinstance(data, dict) and data.get("upstream_path"):
            return str(data["upstream_path"])
    return None


def directory_of(upstream: str) -> str:
    parts = [p for p in upstream.split("/") if p]
    for i, p in enumerate(parts):
        if p in ANCHORS and i + 1 < len(parts):
            return parts[i + 1]
    return ""


def mapping() -> tuple[dict[str, list[str]], list[str]]:
    sources = ledger_sources()
    by_source: dict[str, list[str]] = defaultdict(list)
    unmapped: list[str] = []
    for pkg in sorted(PACKAGES.iterdir()):
        if not pkg.is_dir():
            continue
        name = pkg.name
        if name in CHARACTER_PACKAGES:
            continue
        hit = EXPLICIT.get(name)
        if not hit:
            upstream = upstream_path(pkg)
            if upstream:
                directory = directory_of(upstream)
                repo = "noname-liuli" if name.endswith("-liuli") else ("noname" if name.endswith("-noname") else "")
                for sid, (_nm, srepo, sdir) in sources.items():
                    if sdir == directory and (not repo or srepo == repo):
                        hit = sid
                        break
                if not hit:
                    for sid, (_nm, _r, sdir) in sources.items():
                        if sdir == directory:
                            hit = sid
                            break
        if hit:
            by_source[hit].append(name)
        else:
            unmapped.append(name)
    return dict(by_source), unmapped


def main(argv: list[str]) -> int:
    sources = ledger_sources()
    by_source, unmapped = mapping()
    rows = ["来源\t名称\t包数\t包"]
    for sid in sorted(by_source):
        rows.append(f"{sid}\t{sources[sid][0]}\t{len(by_source[sid])}\t{','.join(sorted(by_source[sid]))}")
    text = "\n".join(rows) + "\n"

    if "--tsv" in argv:
        Path(argv[argv.index("--tsv") + 1]).write_text(text, encoding="utf-8")
        print(f"写入 {argv[argv.index('--tsv') + 1]}")

    print(f"包 {sum(len(v) for v in by_source.values())}；映射到来源 {len(by_source)}；未映射 {len(unmapped)}")
    print(f"无包来源 {len(set(sources) - set(by_source))}: {sorted(set(sources) - set(by_source))}")
    if unmapped:
        print("未映射的包:", unmapped)

    if "--check" in argv:
        ok = not unmapped and len(by_source) == REQUIRED and set(sources) - set(by_source) == NO_PACKAGE
        print("映射" + ("符合预期" if ok else "与预期不符"))
        return 0 if ok else 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
