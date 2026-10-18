#!/usr/bin/env python3
"""主稿本地链接与锚点检查（NC11 证据工具，只读）。

用法（仓库根目录）：

    python3 -B docs/operations/evidence/2026-10-18-nc11-docs-convergence/check_links.py \
        docs/architecture/native-progress.md docs/migration/upstream-extension-reconstruction.md \
        docs/architecture/native-rules-baseline.md docs/native-coverage-manifest.md

判据：
- 只检查本地目标（跳过 http/https/mailto 与空链接）；`#anchor` 检查同文件锚点。
- 目标是 Markdown 时同时校验锚点：显式 `<a id="...">`/`<a name="...">` 或标题 slug
  （GitHub 风格：小写、去标点、空格转连字符，保留中文）。
- 目标不是 Markdown 时只校验文件存在，并打印其类型以便人工确认。
- 输出每个文件的问题清单；存在坏链/坏锚点时 exit 1。
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
from pathlib import Path


INLINE_LINK = re.compile(r"(?<!\!)\[[^\]]*\]\(([^()\s]*(?:\([^()]*\)[^()\s]*)*)\)")
REF_LINK = re.compile(r"^\s*\[[^\]]+\]:\s*(\S+)")
EXPLICIT_ANCHOR = re.compile(r"<a\s+(?:id|name)=\"([^\"]+)\"", re.IGNORECASE)
HEADING = re.compile(r"^(#{1,6})\s+(.*?)\s*$")
HTML_TAG = re.compile(r"<[^>]+>")
CODE_FENCE = re.compile(r"^\s*(```|~~~)")


def slug(text: str) -> str:
    text = HTML_TAG.sub("", text)
    text = text.replace("`", "").strip().lower()
    kept = []
    for char in text:
        if char in "-_":
            kept.append(char)
            continue
        if char.isspace():
            kept.append(" ")
            continue
        category = unicodedata.category(char)
        if category[0] in "LN" or category == "Po" and char in "．":
            kept.append(char)
    result = "".join(kept)
    # GitHub 语义：每个空格转一个连字符，不折叠连续空格产生的 `--`。
    result = result.replace(" ", "-")
    return result.strip("-")


def anchors_of(path: Path) -> set[str]:
    anchors: set[str] = set()
    counts: dict[str, int] = {}
    in_fence = False
    for line in path.read_text(encoding="utf-8", errors="replace").split("\n"):
        if CODE_FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        anchors.update(EXPLICIT_ANCHOR.findall(line))
        heading = HEADING.match(line)
        if not heading:
            continue
        base = slug(heading.group(2))
        if not base:
            continue
        if base in counts:
            counts[base] += 1
            anchors.add(f"{base}-{counts[base]}")
        else:
            counts[base] = 0
            anchors.add(base)
    return anchors


def iter_links(path: Path):
    in_fence = False
    for number, line in enumerate(path.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
        if CODE_FENCE.match(line):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for match in INLINE_LINK.finditer(line):
            yield number, match.group(1)
        reference = REF_LINK.match(line)
        if reference:
            yield number, reference.group(1)


def clean_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if " " in target:
        target = target.split(" ", 1)[0]
    return target


def check_file(path: Path, root: Path) -> list[dict]:
    problems: list[dict] = []
    if not path.exists():
        return [{"file": str(path), "line": 0, "target": "", "reason": "source_missing"}]
    for line, raw in iter_links(path):
        target = clean_target(raw)
        if not target or target.startswith(("http://", "https://", "mailto:", "tel:")):
            continue
        if target.startswith("#"):
            anchor = target[1:]
            if anchor and anchor not in anchors_of(path):
                problems.append(
                    {"file": str(path), "line": line, "target": target, "reason": "anchor_missing"}
                )
            continue
        path_part, _, anchor = target.partition("#")
        resolved = (path.parent / path_part).resolve()
        if not resolved.exists():
            problems.append(
                {"file": str(path), "line": line, "target": target, "reason": "target_missing"}
            )
            continue
        if anchor and resolved.is_file() and resolved.suffix.lower() == ".md":
            if anchor not in anchors_of(resolved):
                problems.append(
                    {
                        "file": str(path),
                        "line": line,
                        "target": target,
                        "reason": "anchor_missing",
                        "resolved": display_path(resolved, root),
                    }
                )
    return problems


def display_path(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", nargs="+", type=Path)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    root = args.root.resolve()
    problems: list[dict] = []
    checked = []
    for path in args.files:
        resolved = path if path.is_absolute() else (root / path)
        checked.append(str(resolved.relative_to(root)) if resolved.is_relative_to(root) else str(resolved))
        problems.extend(check_file(resolved, root))
    if args.json:
        print(json.dumps({"checked": checked, "problems": problems}, ensure_ascii=False, indent=1))
    else:
        for item in problems:
            print(
                "%s:%s -> %s [%s]"
                % (item["file"], item["line"], item["target"], item["reason"])
            )
        print("checked %d files, %d link problems" % (len(checked), len(problems)))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
