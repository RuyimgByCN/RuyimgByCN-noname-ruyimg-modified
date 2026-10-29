#!/usr/bin/env python3
"""NC08 方案 A 差分统计·核心武将包扩展（裁决第 14 项，只统计不改语义）。

把 2026-10-28 的 37 来源扩展面统计（../2026-10-28-nc08-plan-a-statistics/）
扩展到**两仓核心武将包**：noname `apps/core/character/`（子目录各为一包，
另含顶层散件）与 noname-liuli `noname/character/`（每 .js 一包）。口径、
优先级公式与解析器全部复用原 scan.py（import 复用，不复制实现）。

另导出每个命中组成员技能的 filter 源片段（截断），供逐组互斥核对。
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "2026-10-28-nc08-plan-a-statistics"))
import scan as base  # noqa: E402

REPO = HERE.parents[3]           # noname-next/
UP = REPO.parent / "archive/upstream"
CORE_MAIN = UP / "noname/apps/core/character"
CORE_LIULI = UP / "noname-liuli/noname/character"


def scan_files(files: list[Path]) -> dict:
    """base.scan_source 的文件清单版（liuli 每包=目录中一个 .js，按目录扫会
    把全部包算进每一包）。解析逻辑复用 base 的内部函数。"""
    import re as _re
    skills: dict[str, dict] = {}
    translates: set[str] = set()
    generals: dict[str, list[str]] = {}
    for path in files:
        try:
            src = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        m = base.mask(src)
        for mt in _re.finditer(r"\n\s*([A-Za-z_$][\w$]*)\s*:\s*\{", m):
            name = mt.group(1)
            if name in base.NON_SKILL_KEYS:
                continue
            try:
                (lo, hi), _ = base.balanced(m, mt.end() - 1)
            except ValueError:
                continue
            entries = base.top_level_entries(m, src, lo, hi)
            if "trigger" in entries:
                skills.setdefault(name, base.parse_skill(name, m, src, lo, hi))
            elif "skills" in entries and name not in ("character", "card"):
                span = entries["skills"]
                lst = _re.findall(r"['\"](\w+)['\"]", src[span[0]:span[1]])
                if lst:
                    generals.setdefault(name, lst)
        for ms in _re.finditer(r"lib\.skill(?:\[(['\"])(\w+)\1\]|\.(\w+))\s*=\s*\{", m):
            name = ms.group(2) or ms.group(3)
            if name and name not in skills:
                try:
                    (lo, hi), _ = base.balanced(m, ms.end() - 1)
                except ValueError:
                    continue
                if "trigger" in base.top_level_entries(m, src, lo, hi):
                    skills[name] = base.parse_skill(name, m, src, lo, hi)
        for mt in _re.finditer(r"\b(?:translates|translate)\s*[:=]\s*\{", m):
            try:
                (lo, hi), _ = base.balanced(m, mt.end() - 1)
            except ValueError:
                continue
            translates.update(base.top_level_entries(m, src, lo, hi).keys())
        for mt in _re.finditer(r"lib\.translate(?:\[(['\"])(\w+)\1\]|\.(\w+))\s*=", m):
            translates.add(mt.group(2) or mt.group(3))
    return {"skills": skills, "translates": translates, "generals": generals,
            "files": len(files)}


def core_sources() -> list[tuple[str, list[Path]]]:
    """（标签, 该包的文件清单）。noname/character 子目录=一包（含嵌套）；
    liuli/character 每个 .js=一包（排除 clan.js 等非武将散件）。"""
    out: list[tuple[str, list[Path]]] = []
    if CORE_MAIN.is_dir():
        for p in sorted(CORE_MAIN.iterdir()):
            if p.is_dir():
                out.append((f"noname/character/{p.name}",
                            [q for q in p.rglob("*.js") if q.name not in base.SKIP_FILES
                             and "image" not in q.parts]))
            elif p.suffix == ".js" and p.stem not in ("rank", "replace", "perfectPairs"):
                out.append((f"noname/character/{p.stem}", [p]))
    if CORE_LIULI.is_dir():
        for p in sorted(CORE_LIULI.glob("*.js")):
            out.append((f"noname-liuli/character/{p.stem}", [p]))
    return out


def filter_snippets(files: list[Path], names: set[str]) -> dict[str, str]:
    """提取命中技能的 filter 源（首个定义处，截断 600 字符）。"""
    out: dict[str, str] = {}
    for path in files:
        if len(out) == len(names):
            break
        try:
            src = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        m = base.mask(src)
        for name in names - out.keys():
            for mt in re.finditer(rf"\n\s*{re.escape(name)}\s*:\s*\{{", m):
                brace = mt.end() - 1  # 匹配止于 {
                try:
                    (lo, hi), _ = base.balanced(m, brace)
                except ValueError:
                    continue
                if "trigger" not in base.top_level_entries(m, src, lo, hi):
                    continue
                # filter 可为 `filter: {...}` 键或 `filter(args){...}` 方法简写
                # （上游普遍用简写，无冒号——top_level_entries 抓不到）。
                fm = re.compile(r"filter\s*(?:\([^)]*\))?\s*(?:=>)?\s*\{").search(m, lo, hi)
                if not fm:
                    out[name] = "（无 filter：锁定直触发）"
                    break
                try:
                    (flo, fhi), _ = base.balanced(m, fm.end() - 1)
                except ValueError:
                    out[name] = "（filter 块不平衡）"
                    break
                snippet = src[flo:fhi].strip()
                out[name] = snippet if len(snippet) <= 600 else snippet[:600] + " …"
                break
    return out


def main() -> None:
    report = {"sources": [], "totals": {}}
    all_hits: list[dict] = []
    for label, files in core_sources():
        data = scan_files(files)
        hits = base.hits_of(label, data)
        need = {s for h in hits for s in h["skills"]}
        snips = filter_snippets(files, need)
        for h in hits:
            h["source"] = label
            h["filters"] = {k: snips.get(k, "（本包未内联 filter：见 translate/共享定义）") for k in h["skills"]}
        all_hits.extend(hits)
        report["sources"].append({
            "source": label, "files": data["files"], "skills": len(data["skills"]),
            "skills_with_trigger": sum(1 for s in data["skills"].values() if s["events"]),
            "generals": len(data["generals"]), "translates": len(data["translates"]),
            "hit_groups": len(hits),
            "hit_generals": len({h["general"] for h in hits}),
        })
    report["totals"] = {
        "sources_scanned": len(report["sources"]),
        "skills_total": sum(s["skills"] for s in report["sources"]),
        "skills_with_trigger_total": sum(s["skills_with_trigger"] for s in report["sources"]),
        "generals_total": sum(s["generals"] for s in report["sources"]),
        "hit_groups": len(all_hits),
        "hit_generals": len({(h["source"], h["general"]) for h in all_hits}),
        "same_scope_groups": sum(1 for h in all_hits if h["same_scope"]),
        "forced_involving_groups": sum(1 for h in all_hits if h["any_forced"]),
        "pairs": sum(len(h["skills"]) * (len(h["skills"]) - 1) // 2 for h in all_hits),
    }
    out = HERE
    (out / "hits-core.json").write_text(
        json.dumps({"report": report, "hits": all_hits}, ensure_ascii=False, indent=1),
        encoding="utf-8")
    for s in report["sources"]:
        print(f"{s['source']}: files={s['files']} skills={s['skills']}"
              f"(trigger={s['skills_with_trigger']}) generals={s['generals']} hit_groups={s['hit_groups']}")
    print(json.dumps(report["totals"], ensure_ascii=False, indent=1))


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()
