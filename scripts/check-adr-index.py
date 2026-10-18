#!/usr/bin/env python3
"""Check ADR numbering: duplicate numbers, wrong heading numbers and INDEX coverage.

用法（仓库根目录）：

    python3 -B scripts/check-adr-index.py            # 人类可读报告，失败 exit 1
    python3 -B scripts/check-adr-index.py --json     # 机器可读输出

判据（NC11「ADR 重复编号」要求：维护唯一标识、阻止新增冲突，不批量删除历史 ADR）：

1. 文件名 `NNNN-主题.md` 的前四位是编号；首个标题中的四位数字应与文件名一致
   （标题错号 / heading-id mismatch）。
2. 同一编号被多个文件共用即冲突。历史冲突不重命名、不删除，但**必须**登记在
   `docs/adr/INDEX.md` 的「编号冲突消歧表」中；出现未登记的新冲突即失败。
3. 消歧表里的编号分组必须与磁盘实测一致（多余的过时登记也报错，避免表与文件漂移）。
4. 消歧表与索引清单中的相对链接必须存在。

脚本只读，不修改任何 ADR；修正方式见 INDEX「维护约定」。
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
ADR_DIR = REPO_ROOT / "docs" / "adr"
INDEX = ADR_DIR / "INDEX.md"

NUMBERED = re.compile(r"^(\d{4})-")
HEADING = re.compile(r"^#{1,6}\s+(.*)$")
HEADING_NUMBER = re.compile(r"(\d{4})")
TABLE_ROW = re.compile(r"^\|\s*(\d{4})\s*\|(.*)\|\s*$")
CODE_SPAN = re.compile(r"`([^`]+)`")
MD_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")


class Report:
    def __init__(self) -> None:
        self.adr_files: list[str] = []
        self.numbered_files = 0
        self.unique_numbers = 0
        self.duplicates: dict[str, list[str]] = {}
        self.heading_mismatches: list[dict[str, str]] = []
        self.registry_duplicates: dict[str, list[str]] = {}
        self.registry_missing: list[str] = []  # 磁盘有冲突、消歧表未登记
        self.registry_stale: list[str] = []  # 消歧表有分组、磁盘已无冲突
        self.registry_mismatch: list[dict[str, object]] = []
        self.link_missing: list[dict[str, str]] = []

    @property
    def failures(self) -> int:
        return (
            len(self.heading_mismatches)
            + len(self.registry_missing)
            + len(self.registry_stale)
            + len(self.registry_mismatch)
            + len(self.link_missing)
        )

    def as_json(self) -> dict:
        return {
            "adr_dir": str(ADR_DIR.relative_to(REPO_ROOT)),
            "adr_files": len(self.adr_files),
            "numbered_files": self.numbered_files,
            "unique_numbers": self.unique_numbers,
            "duplicate_groups": len(self.duplicates),
            "duplicate_files": sum(len(v) for v in self.duplicates.values()),
            "duplicates": self.duplicates,
            "heading_mismatches": self.heading_mismatches,
            "registry_duplicate_groups": len(self.registry_duplicates),
            "registry_missing": self.registry_missing,
            "registry_stale": self.registry_stale,
            "registry_mismatch": self.registry_mismatch,
            "link_missing": self.link_missing,
            "failures": self.failures,
        }


def first_heading_id(text: str) -> str | None:
    for line in text.split("\n"):
        match = HEADING.match(line)
        if not match:
            continue
        number = HEADING_NUMBER.search(match.group(1))
        return number.group(1) if number else None
    return None


def scan_adr_dir(adr_dir: Path) -> tuple[dict[str, list[str]], list[dict[str, str]], int]:
    by_number: dict[str, list[str]] = defaultdict(list)
    mismatches: list[dict[str, str]] = []
    numbered = 0
    for path in sorted(adr_dir.glob("*.md")):
        match = NUMBERED.match(path.name)
        if not match:
            continue
        numbered += 1
        number = match.group(1)
        by_number[number].append(path.name)
        heading_id = first_heading_id(path.read_text(encoding="utf-8", errors="replace"))
        if heading_id is not None and heading_id != number:
            mismatches.append(
                {
                    "file": path.name,
                    "filename_id": number,
                    "heading_id": heading_id,
                }
            )
    return dict(by_number), mismatches, numbered


def parse_registry(index_text: str) -> dict[str, list[str]]:
    """读取「编号冲突消歧表」的编号 → 文件清单（反引号内的文件名）。"""
    registry: dict[str, list[str]] = {}
    for line in index_text.split("\n"):
        row = TABLE_ROW.match(line)
        if not row:
            continue
        number = row.group(1)
        files = [name for name in CODE_SPAN.findall(row.group(2)) if name.endswith(".md")]
        if not files:
            continue
        registry[number] = sorted(files)
    return registry


def _link_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if " " in target and not target.startswith("<"):
        target = target.split(" ", 1)[0]
    return target


def check_index_links(index_text: str, adr_dir: Path) -> list[dict[str, str]]:
    missing: list[dict[str, str]] = []
    for match in MD_LINK.finditer(index_text):
        target = _link_target(match.group(1))
        if target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        path_part = target.split("#", 1)[0]
        if not path_part or path_part.startswith("/"):
            continue
        resolved = (adr_dir / path_part).resolve()
        if not resolved.exists():
            missing.append({"index": "docs/adr/INDEX.md", "target": target})
    return missing


def build_report(adr_dir: Path, index_path: Path) -> Report:
    report = Report()
    by_number, mismatches, numbered = scan_adr_dir(adr_dir)
    index_text = index_path.read_text(encoding="utf-8") if index_path.exists() else ""
    report.adr_files = sorted(path.name for path in adr_dir.glob("*.md"))
    report.numbered_files = numbered
    report.unique_numbers = len(by_number)
    report.duplicates = {key: value for key, value in sorted(by_number.items()) if len(value) > 1}
    report.heading_mismatches = mismatches
    registry = parse_registry(index_text)
    report.registry_duplicates = registry
    report.registry_missing = sorted(set(report.duplicates) - set(registry))
    report.registry_stale = sorted(set(registry) - set(report.duplicates))
    for number in sorted(set(registry) & set(report.duplicates)):
        on_disk = sorted(report.duplicates[number])
        registered = registry[number]
        if on_disk != registered:
            report.registry_mismatch.append(
                {"number": number, "on_disk": on_disk, "registry": registered}
            )
    report.link_missing = check_index_links(index_text, adr_dir)
    return report


def render(report: Report) -> str:
    lines = [
        "ADR 编号检查（scripts/check-adr-index.py）",
        f"- ADR 文件 {len(report.adr_files)}（编号文件 {report.numbered_files}，唯一编号 {report.unique_numbers}）",
        f"- 重复编号 {len(report.duplicates)} 组 / {sum(len(v) for v in report.duplicates.values())} 个文件",
        f"- 消歧表登记 {len(report.registry_duplicates)} 组",
        f"- 标题错号 {len(report.heading_mismatches)}",
        f"- 未登记冲突 {len(report.registry_missing)}，过时登记 {len(report.registry_stale)}，"
        f"登记内容不符 {len(report.registry_mismatch)}，索引坏链 {len(report.link_missing)}",
    ]
    for item in report.heading_mismatches:
        lines.append(
            f"  [标题错号] {item['file']}：文件名 {item['filename_id']} / 标题 {item['heading_id']}"
        )
    for number in report.registry_missing:
        lines.append(f"  [未登记冲突] {number}: {', '.join(report.duplicates[number])}")
    for number in report.registry_stale:
        lines.append(f"  [过时登记] {number}: 磁盘已无冲突，请从消歧表移除")
    for item in report.registry_mismatch:
        lines.append(
            f"  [登记不符] {item['number']}: 磁盘 {item['on_disk']} / 表 {item['registry']}"
        )
    for item in report.link_missing:
        lines.append(f"  [索引坏链] {item['index']} -> {item['target']}")
    lines.append("检查结果：%s" % ("通过" if report.failures == 0 else "失败 %d 项" % report.failures))
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--adr-dir", type=Path, default=ADR_DIR)
    parser.add_argument("--index", type=Path, default=None)
    parser.add_argument("--json", action="store_true", help="输出机器可读 JSON")
    args = parser.parse_args(argv)
    adr_dir = args.adr_dir
    index_path = args.index or (adr_dir / "INDEX.md")
    report = build_report(adr_dir, index_path)
    if args.json:
        print(json.dumps(report.as_json(), ensure_ascii=False, indent=1, sort_keys=True))
    else:
        print(render(report))
    return 1 if report.failures else 0


if __name__ == "__main__":
    sys.exit(main())
