#!/usr/bin/env python3
"""Verify `path/to/file.rs:123` references in docs still point inside the file.

Evidence documents cite exact source lines (`step/skills_cards.rs:2197`).  The
code they cite is **untracked** and being edited by other agents, so a citation
can silently go stale — pointing past the end of a shrunken file, or at a file
that moved.

This check is deliberately shallow: it verifies the file exists and has at least
as many lines as the citation claims.  It **cannot** verify the line still means
what the doc says — that would need semantic matching, and pretending otherwise
would be worse than not checking.
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"

#: `xxx.rs:123`, ranges with ASCII/Unicode dashes, or slash-separated lines.
REF_RE = re.compile(r"([A-Za-z0-9_./-]+\.(?:rs|py|ts|tsx|js|json|toml|md)):(\d+)((?:[-–—/]\d+)*)")

#: 少见的合法目标（不在 docs 的父目录树里）。
SKIP_PREFIXES = ("http", "www.")


def doc_files() -> list[Path]:
    return [path for path in DOCS.rglob("*.md") if path.is_file()]


_INDEX: dict[str, list[Path]] = {}


def _index() -> dict[str, list[Path]]:
    """basename -> candidate paths (built once; rglob over the archive is slow)."""
    if _INDEX:
        return _INDEX
    for base in (ROOT, ROOT.parent / "archive"):
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if path.is_file() and path.suffix in {".rs", ".py", ".ts", ".tsx", ".js", ".json", ".toml", ".md"}:
                _INDEX.setdefault(path.name, []).append(path)
    return _INDEX


def candidates(candidate: str) -> list[Path]:
    """All files a doc citation could mean (repo first, then the sibling archive)."""
    if candidate.startswith(SKIP_PREFIXES):
        return []
    direct = (ROOT / candidate).resolve()
    if direct.is_file():
        return [direct]
    return [
        path
        for path in _index().get(Path(candidate).name, [])
        if str(path).replace("\\", "/").endswith(candidate)
    ]


def resolve(candidate: str) -> Path | None:
    """Resolve a doc citation to a repo file, tolerating a path suffix."""
    if candidate.startswith(SKIP_PREFIXES):
        return None
    direct = (ROOT / candidate).resolve()
    if direct.is_file():
        return direct
    # 上游快照在**仓库父目录**的 `archive/` 下（`noname-next` 的兄弟目录）
    for base in (ROOT, ROOT.parent / "archive"):
        if not base.is_dir():
            continue
        for path in base.rglob(Path(candidate).name):
            if str(path).replace("\\", "/").endswith(candidate):
                return path.resolve()
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--strict", action="store_true", help="exit 1 when a citation is stale")
    args = ap.parse_args()

    problems: list[str] = []
    checked = 0
    missing_files: Counter[str] = Counter()
    for doc in doc_files():
        text = doc.read_text(encoding="utf-8", errors="replace")
        for match in REF_RE.finditer(text):
            candidate, start, end = match.group(1), int(match.group(2)), match.group(3)
            options = candidates(candidate)
            checked += 1
            if not options:
                missing_files[candidate] += 1
                continue
            last = max([start, *map(int, re.findall(r"\d+", end or ""))])
            # 同名文件可能有多个（仓库 + 上游快照）；取**行数足够**且最大的那个
            sizes: list[tuple[int, Path]] = []
            for option in options:
                try:
                    total = sum(1 for _ in option.open(encoding="utf-8", errors="replace"))
                except OSError:
                    continue
                sizes.append((total, option))
            fitting = [(total, option) for total, option in sizes if total >= last]
            if fitting:
                continue
            biggest = max(sizes, default=(0, options[0]))
            problems.append(
                f"{doc.relative_to(ROOT)}: {candidate}:{last} 超出所有候选的最大行数 {biggest[0]}"
            )

    print(f"检查 {checked} 处 `file:line` 引用；文件不存在 {len(missing_files)} 种；越界 {len(problems)} 处")
    if missing_files:
        print("\n找不到的文件（可能是文档里的示例路径）：", file=sys.stderr)
        for name, count in missing_files.most_common(10):
            print(f"  - {name}（{count} 处）", file=sys.stderr)
    if problems:
        print("\n越界的引用：", file=sys.stderr)
        for problem in problems[:20]:
            print(f"  - {problem}", file=sys.stderr)
        return 1 if args.strict else 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
