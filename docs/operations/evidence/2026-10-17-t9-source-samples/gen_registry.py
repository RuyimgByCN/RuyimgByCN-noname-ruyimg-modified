#!/usr/bin/env python3
"""重建「T9 来源证据关系登记表」（台账同源解析 + 机器可读字段）。

用法（在 noname-next/ 根目录）：

    python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py
    python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py --fail-on-missing

本脚本是 NC11（2026-10-18 验收数据与文档收敛）升级后的证据模型实现，替换原先
「台账状态 / 证据目录 / 测试文件」三列版本：

    来源功能·配置入口 → 上游证据 → 生产接口 → 测试案例 → 源码/规则/平台版本 → 结果 → 未覆盖项

口径（对应方案 NC11 与 AGENTS.md）：

- **同源**：来源行、状态标签、证据目录、测试名、包目录全部从
  `docs/migration/upstream-extension-reconstruction.md` 逐行解析；整包状态与
  `docs/operations/evidence/2026-10-16-todo-register/todo-register.json` 的机器字段
  （tags 含「整包」的条目 status）绑定，不另建第二套人工状态表。
- **禁止自动认证完成**：脚本只做「引用是否存在」与「是否登记未覆盖项」的实测，
  **不输出完成/未完成结论**。匹配「八项整包」字符串、包目录存在或测试文件存在，
  都不足以判定整包完成（方案 NC11 明确禁止）；因此脚本额外产出**证据缺失清单**
  （缺生产接口 / 缺测试案例 / 缺平台结果的来源逐条列出）。
- **平台结果判据**：行内或其证据目录中出现 wasm／浏览器／冒烟／真实客户端／平台／
  Tauri／Electron／Android／iOS／Windows／macOS 等关键词记为「有平台证据线索」，
  否则记为「缺平台结果」。这是关键词判据，不是平台验收结论。
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
LEDGER = ROOT / "docs/migration/upstream-extension-reconstruction.md"
REGISTER = ROOT / "docs/operations/evidence/2026-10-16-todo-register/todo-register.json"
EVIDENCE = ROOT / "docs/operations/evidence"
PACKAGES = ROOT / "packages/extensions"
G0_DIR = ROOT / "docs/migration/g0"
TEST_ROOTS = (
    ROOT / "crates/engine-native/tests",
    ROOT / "crates/content-native-loader/tests",
    ROOT / "apps/game-server/tests",
    ROOT / "crates/engine-native-wasm/tests",
)

ROW_START = re.compile(r"^\|\s*(U\d+|L\d+)\s*\[([^\]]+)\]\((.*?)\)\s*\|")
STATUS_LABELS = ("八项整包", "部分重构", "G0 完成", "待重构")
EVIDENCE_DIR = re.compile(r"(?:\.\./)+operations/evidence/([0-9A-Za-z._-]+)/")
PACKAGE_DIR = re.compile(r"`packages/extensions/([0-9A-Za-z._-]+)`")
TEST_FILE = re.compile(r"`([0-9A-Za-z_./-]+\.rs)`")
ANY_RUST_FILE = re.compile(r"\b([0-9A-Za-z_./-]+\.rs)\b")
LOADER_MODULE = re.compile(r"content[_-]native[_-]loader::([0-9A-Za-z_]+)")
PLATFORM_HINT = re.compile(
    r"wasm|WASM|浏览器|冒烟|真实客户端|客户端|平台|Tauri|Electron|Android|iOS|Windows|macOS|"
    r"真机|设备|smoke"
)
COVERAGE_MARKERS = (
    "登记差异",
    "未迁移",
    "不迁移",
    "deferred",
    "暂缓",
    "未实现",
    "阻塞",
    "未验收",
    "递延",
)
UNMIGRATED_SURFACE = re.compile(
    r"画界面|不画|显示面|界面|UI|音频|语音|动画|渲染|DOM|窗口|平台|联机|录像|帮助|"
    r"Spine|插件系统|外部依赖|外部技|结构化阻塞|阻塞"
)
GAP_ORDER = ("上游证据", "生产接口", "测试案例", "平台结果")


def git_head() -> str:
    try:
        return subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
    except Exception:  # pragma: no cover - 仅用于报告头
        return "unknown"


def dirty_summary() -> str:
    try:
        out = subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        return "%d 项未提交改动" % len([line for line in out.split("\n") if line.strip()])
    except Exception:  # pragma: no cover
        return "未取到"


def source_version() -> dict:
    """从源码读取规则/存档版本常量（实测，不手写）。"""
    versions = {"ruleset": None, "archive": None, "ruleset_source": "", "archive_source": ""}
    ruleset = ROOT / "crates/engine-native/src/content/standard.rs"
    archive = ROOT / "crates/engine-native/src/archive.rs"
    for key, path, pattern in (
        ("ruleset", ruleset, r"pub const STANDARD_RULESET_VERSION: u64 = (\d+);"),
        ("archive", archive, r"pub const ARCHIVE_FORMAT_VERSION: u16 = (\d+);"),
    ):
        text = path.read_text(encoding="utf-8", errors="replace")
        match = re.search(pattern, text)
        if not match:
            continue
        versions[key] = int(match.group(1))
        line = text[: match.start()].count("\n") + 1
        versions[key + "_source"] = "%s:%d" % (path.relative_to(ROOT), line)
    return versions


def register_wholepack() -> dict:
    """登记表机器字段：tags 含「整包」的条目按 status 归类。"""
    data = json.loads(REGISTER.read_text(encoding="utf-8"))
    items = [item for item in data["items"] if "整包" in (item.get("tags") or [])]
    by_source = {}
    for item in items:
        source = next(
            (tag for tag in (item.get("tags") or []) if re.fullmatch(r"[UL]\d\d", tag)),
            item.get("id", ""),
        )
        by_source[source] = item.get("status")
    resolved = sorted(source for source, status in by_source.items() if status == "resolved")
    open_items = sorted(source for source, status in by_source.items() if status != "resolved")
    return {
        "item_count": data.get("meta", {}).get("item_count"),
        "resolved_count": data.get("meta", {}).get("resolved_count"),
        "open_count": data.get("meta", {}).get("open_count"),
        "wholepack_count": data.get("meta", {}).get("wholepack_count"),
        "wholepack_items": len(items),
        "resolved_sources": resolved,
        "open_sources": open_items,
        "by_source": by_source,
    }


def parse_ledger() -> list[dict]:
    rows = []
    for line in LEDGER.read_text(encoding="utf-8").split("\n"):
        match = ROW_START.match(line)
        if not match:
            continue
        source_id, name, upstream_link = match.group(1), match.group(2), match.group(3).strip()
        if upstream_link.startswith("<") and upstream_link.endswith(">"):
            upstream_link = upstream_link[1:-1]
        cells = line.strip().strip("|").split(" | ")
        if len(cells) != 5:
            cells = [cells[0], " ".join(cells[1:-1]), cells[-1]]
        label = next((item for item in STATUS_LABELS if item in line), "其他")
        rows.append(
            {
                "id": source_id,
                "name": name,
                "upstream": upstream_link,
                "cells": cells,
                "text": line,
                "label": label,
            }
        )
    return rows


def package_index() -> dict:
    index = {}
    for manifest in sorted(PACKAGES.glob("*/package.npack.json")):
        data = json.loads(manifest.read_text(encoding="utf-8"))
        meta = data.get("manifest", {})
        index[manifest.parent.name] = {
            "namespace": meta.get("namespace"),
            "package_id": meta.get("package_id"),
            "version": meta.get("version"),
            "path": str(manifest.relative_to(ROOT)),
        }
    return index


def test_index() -> dict:
    index = {}
    for base in TEST_ROOTS:
        if not base.exists():
            continue
        for path in sorted(base.rglob("*.rs")):
            index.setdefault(path.name, str(path.relative_to(ROOT)))
    return index


def loader_modules() -> set:
    base = ROOT / "crates/content-native-loader/src"
    return {path.stem for path in base.rglob("*.rs")} if base.exists() else set()


def evidence_dirs() -> set:
    return {path.name for path in EVIDENCE.iterdir() if path.is_dir()}


def tests_from_evidence(ev_dirs: list[str], tests: dict) -> list[str]:
    """证据目录 README/日志里出现的、仓库中确实存在的测试文件名。"""
    found = []
    for name in ev_dirs:
        for path in sorted((EVIDENCE / name).glob("**/*")):
            if not path.is_file() or path.suffix not in (".md", ".txt", ".log", ".json"):
                continue
            text = path.read_text(encoding="utf-8", errors="replace")[:20000]
            for candidate in ANY_RUST_FILE.findall(text):
                base = Path(candidate).name
                if base in tests:
                    found.append(base)
    return sorted(set(found))


def has_platform_hint(row: dict, ev_dirs: list[str], ev_names: set) -> bool:
    if PLATFORM_HINT.search(row["text"]):
        return True
    for name in ev_dirs:
        if PLATFORM_HINT.search(name):
            return True
        readme = EVIDENCE / name / "README.md"
        if readme.exists():
            head = readme.read_text(encoding="utf-8", errors="replace")[:4000]
            if PLATFORM_HINT.search(head):
                return True
    return False


def enrich(rows: list[dict], register: dict, packages: dict, tests: dict, modules: set, ev_names: set):
    for row in rows:
        text = row["text"]
        row["evidence"] = sorted(set(EVIDENCE_DIR.findall(text)))
        row["missing_evidence"] = [name for name in row["evidence"] if name not in ev_names]
        row["packages"] = sorted(set(PACKAGE_DIR.findall(text)))
        # 台账行也常用 `namespace/package` 或裸包名指代转换包：按在盘包目录名再核一遍。
        for dirname in packages:
            if re.search(r"`[^`]*%s[^`]*`" % re.escape(dirname), text):
                row["packages"] = sorted(set(row["packages"]) | {dirname})
        row["missing_packages"] = [
            name for name in row["packages"] if name not in packages
        ]
        row["loader"] = sorted(set(LOADER_MODULE.findall(text)))
        row["missing_loader"] = [name for name in row["loader"] if name not in modules]
        row["tests"] = sorted(set(TEST_FILE.findall(text)))
        row["missing_tests"] = [
            name for name in row["tests"] if Path(name).name not in tests
        ]
        row["tests_evidence"] = [
            name
            for name in tests_from_evidence(row["evidence"], tests)
            if Path(name).name not in {Path(item).name for item in row["tests"]}
        ]
        row["register_status"] = register["by_source"].get(row["id"], "未登记")
        row["platform"] = has_platform_hint(row, row["evidence"], ev_names)
        row["uncovered"] = sorted({marker for marker in COVERAGE_MARKERS if marker in text})
        row["surface"] = sorted(set(UNMIGRATED_SURFACE.findall(text)))
        gaps = []
        upstream = (ROOT / "docs/migration" / row["upstream"]).resolve()
        row["upstream_exists"] = upstream.exists()
        if not row["upstream_exists"]:
            gaps.append("上游证据")
        if not row["packages"] or row["missing_packages"]:
            gaps.append("生产接口")
        if (not row["tests"] and not row["tests_evidence"]) or row["missing_tests"]:
            gaps.append("测试案例")
        if not row["platform"]:
            gaps.append("平台结果")
        row["gaps"] = [gap for gap in GAP_ORDER if gap in gaps]
        row["suspect_wholepack"] = row["label"] == "八项整包" and bool(row["surface"])
    return rows


def fmt(items, formatter=str, empty="—") -> str:
    return "、".join(formatter(item) for item in items) if items else empty


def plain(text: str) -> str:
    """表格单元里去掉 Markdown 链接目标（相对路径在生成文档里会失效）。"""
    return re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)


def short(text: str, limit: int = 60) -> str:
    text = re.sub(r"\s+", " ", plain(text)).strip()
    return text if len(text) <= limit else text[: limit - 1] + "…"


def render(rows, register, packages, versions, counters) -> str:
    wholepack = [row for row in rows if row["label"] == "八项整包"]
    label_counts = {label: sum(1 for row in rows if row["label"] == label) for label in STATUS_LABELS}
    lines = [
        "# T9 来源证据关系登记表（台账同源解析 · 机器可读字段 · 实测重建）",
        "",
        "- 生成：`python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py`"
        "（可复现；表内计数全部由本脚本实测得出，不手写）。",
        "- 证据关系：**来源功能/配置入口 → 上游证据 → 生产接口 → 测试案例 →"
        " 源码/规则/平台版本 → 结果 → 未覆盖项**。",
        "- 同源绑定：来源行、状态标签、证据目录、包目录、测试名均解析自"
        " [`docs/migration/upstream-extension-reconstruction.md`](../../../migration/upstream-extension-reconstruction.md)；"
        "整包状态取自 [`todo-register.json`](../2026-10-16-todo-register/todo-register.json)"
        " 中 tags 含「整包」条目的 `status` 机器字段，不另建状态表。",
        "- **不自动认证完成**：脚本只报告「引用是否存在 + 是否登记未覆盖项」，不给出完成结论。"
        "匹配「八项整包」字符串、包目录存在或测试文件存在都不足以判定整包完成。",
        "",
        "## 实测计数（%s，HEAD `%s`，%s）"
        % (counters["date"], counters["head"], counters["dirty"]),
        "",
        "| 口径 | 值 |",
        "|---|---|",
        "| 来源实例 | **%d**（U01–U08 + L01–L29） |" % len(rows),
        "| 台账标签 | 八项整包 **%d** / 部分重构 %d / G0 完成 %d / 待重构 %d |"
        % (
            label_counts["八项整包"],
            label_counts["部分重构"],
            label_counts["G0 完成"],
            label_counts["待重构"],
        ),
        "| 登记表整包条目（机器字段） | 已 resolved **%d** / 仍 open **%d**（共 %d 条） |"
        % (
            len(register["resolved_sources"]),
            len(register["open_sources"]),
            register["wholepack_items"],
        ),
        "| 行内引用证据目录 | %d 个引用，缺失 %d |"
        % (sum(len(row["evidence"]) for row in rows), sum(len(row["missing_evidence"]) for row in rows)),
        "| 行内引用转换包 | %d 个引用，缺失 %d（在盘包 **%d** 个，全部含 `package.npack.json`） |"
        % (
            sum(len(row["packages"]) for row in rows),
            sum(len(row["missing_packages"]) for row in rows),
            len(packages),
        ),
        "| 行内引用测试文件 | %d 个引用，缺失 %d；另有 %d 个测试名只在证据目录中出现（未写进台账行） |"
        % (
            sum(len(row["tests"]) for row in rows),
            sum(len(row["missing_tests"]) for row in rows),
            sum(len(row["tests_evidence"]) for row in rows),
        ),
        "| 装载器模块引用 | %d 个，缺失 %d |"
        % (sum(len(row["loader"]) for row in rows), sum(len(row["missing_loader"]) for row in rows)),
        "| 源码版本（实测） | 规则 `STANDARD_RULESET_VERSION = %s`（%s）、存档 `ARCHIVE_FORMAT_VERSION = %s`（%s） |"
        % (
            versions["ruleset"],
            versions["ruleset_source"],
            versions["archive"],
            versions["archive_source"],
        ),
        "| 平台证据线索 | 有 **%d** / 无 **%d**（关键词判据，非平台验收结论） |"
        % (
            sum(1 for row in rows if row["platform"]),
            sum(1 for row in rows if not row["platform"]),
        ),
        "| 证据缺失来源 | 缺上游证据 %d / 缺生产接口 %d / 缺测试案例 %d / 缺平台结果 %d |"
        % tuple(
            sum(1 for row in rows if gap in row["gaps"]) for gap in GAP_ORDER
        ),
        "| 整包标签存疑 | **%d/%d** 个标签行在行内仍登记未迁移功能/表现/平台面 |"
        % (sum(1 for row in wholepack if row["suspect_wholepack"]), len(wholepack)),
        "",
        "## 逐来源证据关系",
        "",
        "| 来源 | 功能/配置入口 | 上游证据 | 生产接口 | 测试案例 | 版本 | 结果 | 未覆盖项 |",
        "|---|---|---|---|---|---|---|---|",
    ]
    for row in sorted(rows, key=lambda item: item["id"]):
        entry = short(row["cells"][1] if len(row["cells"]) > 1 else "", 70)
        upstream = "`%s`%s" % (
            row["upstream"],
            "" if row["upstream_exists"] else "（**缺失**）",
        )
        interface = fmt(
            ["`%s`" % name for name in row["packages"]]
            + ["`loader::%s`" % name for name in row["loader"]]
        )
        cases = fmt(["`%s`" % Path(name).name for name in row["tests"]])
        if row["tests_evidence"]:
            cases += "；证据目录：%s" % fmt(["`%s`" % name for name in row["tests_evidence"]])
        version = "HEAD `%s` / 规则 v%s / 存档 v%s / %s" % (
            counters["head"],
            versions["ruleset"],
            versions["archive"],
            "有平台线索" if row["platform"] else "**缺平台结果**",
        )
        result = "登记表 %s；证据目录 %s" % (
            row["register_status"],
            fmt(["`%s`" % name for name in row["evidence"]]),
        )
        uncovered = fmt(["`%s`" % marker for marker in row["uncovered"]]) if row["uncovered"] else "—"
        if row["suspect_wholepack"]:
            uncovered += "；**标签存疑**：行内仍见 %s" % fmt(row["surface"])
        lines.append(
            "| %s %s | %s | %s | %s | %s | %s | %s | %s |"
            % (row["id"], row["name"], entry, upstream, interface, cases, version, result, uncovered)
        )

    lines += ["", "## 证据缺失清单（脚本实测，逐来源列出）", ""]
    unsupported = []
    for gap in GAP_ORDER:
        hits = [row for row in rows if gap in row["gaps"]]
        lines.append("### 缺%s（%d 个来源）" % (gap, len(hits)))
        lines.append("")
        if not hits:
            lines.append("- 无。")
        for row in hits:
            detail = []
            if gap == "生产接口":
                if not row["packages"]:
                    detail.append("行内未给出转换包目录")
                if row["missing_packages"]:
                    detail.append("目录不存在：%s" % fmt(row["missing_packages"]))
                if not row["loader"]:
                    detail.append("另：行内未给出装载器模块名（数据型包可不要求）")
            elif gap == "测试案例":
                if row["missing_tests"]:
                    detail.append("行内测试文件不存在：%s" % fmt(row["missing_tests"]))
                else:
                    detail.append("行内未给出测试文件名，且引用证据目录中未出现测试名")
            elif gap == "平台结果":
                detail.append("行内与证据目录均无平台/真实客户端关键词线索")
            else:
                detail.append("上游链接不存在：`%s`" % row["upstream"])
            lines.append("- **%s %s**：%s" % (row["id"], row["name"], "；".join(detail)))
        lines.append("")
        unsupported.append("%s %d" % (gap, len(hits)))

    lines += [
        "## 整包标签存疑清单（标签 ≠ 验收）",
        "",
        "以下来源在台账中带「八项整包」标签，但同一行/同一证据内**仍登记未迁移的功能、"
        "表现或平台面**（方案阶段 5 与 AGENTS.md 要求这类面完成验收后才可上调整包）：",
        "",
    ]
    suspects = [row for row in wholepack if row["suspect_wholepack"]]
    for row in suspects:
        lines.append(
            "- **%s %s**（登记表 %s）：行内仍见 %s；未覆盖标记 %s"
            % (
                row["id"],
                row["name"],
                row["register_status"],
                fmt(row["surface"]),
                fmt(list(row["uncovered"])),
            )
        )
    if not suspects:
        lines.append("- 无。")
    lines += [
        "",
        "## 判据与禁止事项",
        "",
        "- 本表**不产出完成结论**：`结果` 列只写登记表状态与证据目录是否在盘；"
        "「八项整包」字符串、包目录或测试文件存在均不构成完成判定（NC11 明文禁止）。",
        "- 「平台结果」为关键词判据（wasm/浏览器/冒烟/真实客户端/平台/设备等），"
        "命中只说明有平台线索，**不等于**平台验收通过。",
        "- 「未覆盖项」只提取行内已登记的标记词（%s 等），"
        "不代表未覆盖面的完整清单；完整清单以各行证据与 G0 文档为准。" % "、".join(COVERAGE_MARKERS),
        "- 标签语义：`G0 完成` 表示**该来源**的原生重构只做到 G0 深度清单（不是「37 来源 G0 清点」"
        "这件事）；`八项整包` 表示台账/登记表把它登记为整包范围，**不表示八项验收已通过**。",
        "- 台账状态或登记表机器字段变化后重跑本脚本，不要手改计数。",
        "",
    ]
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fail-on-missing", action="store_true", help="存在引用缺失时 exit 1")
    parser.add_argument("--date", default="2026-10-18")
    args = parser.parse_args(argv)

    rows = enrich(
        parse_ledger(),
        register_wholepack(),
        package_index(),
        test_index(),
        loader_modules(),
        evidence_dirs(),
    )
    versions = source_version()
    counters = {
        "date": args.date,
        "head": git_head(),
        "dirty": dirty_summary(),
    }
    text = render(rows, register_wholepack(), package_index(), versions, counters)
    (HERE / "README.md").write_text(text, encoding="utf-8")

    missing = sum(
        len(row["missing_evidence"]) + len(row["missing_packages"]) + len(row["missing_tests"])
        + len(row["missing_loader"])
        for row in rows
    )
    gaps = {gap: sum(1 for row in rows if gap in row["gaps"]) for gap in GAP_ORDER}
    print(
        "written %s | 来源 %d | 标签 八项整包 %d | 登记表 resolved %d/open %d | 引用缺失 %d | 证据缺失 %s"
        % (
            HERE / "README.md",
            len(rows),
            sum(1 for row in rows if row["label"] == "八项整包"),
            len(register_wholepack()["resolved_sources"]),
            len(register_wholepack()["open_sources"]),
            missing,
            gaps,
        )
    )
    return 1 if (args.fail_on_missing and missing) else 0


if __name__ == "__main__":
    sys.exit(main())
