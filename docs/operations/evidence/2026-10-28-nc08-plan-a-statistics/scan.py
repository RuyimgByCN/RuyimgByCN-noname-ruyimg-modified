#!/usr/bin/env python3
"""NC08 方案 A 差分统计（不改语义）。

复刻上游 arrangeTrigger（archive/upstream/noname/apps/core/noname/library/
element/content.ts:4016-4062）的询问触发条件，对 37 来源（noname 扩展 8 +
noname-liuli 扩展 29）做静态扫描，统计「同武将、同事件、等有效优先级、
均非 silent、均有翻译、技能名去重后 >1」的技能组——即上游会在窗口内向玩家开
chooseControl（选下一个触发的技能）而原生按 def 序决胜的命中面。

口径与限制（结果解读必读）：
- 优先级公式照抄 game/index.js `_priority`：priority*100（未声明=0）、
  silent +1、equipSkill −25、cardSkill −50、ruleSkill −75。
- 静态扫描不执行 filter/条件（usableSkills 判定），结果是**上界**；
  同事件且 filter 互斥的组合实际不会并列。
- 只统计武将自带技能（character.js / character 块的 skills 表）；
  运行时动态授予/卡牌/装备技能的跨武将并列不在本表内。
- firstDo/lastDo 时机上游不做选择；静态无法区分时机类别，逐组列出供人工核对。

解析方式：`mask()` 生成与原文等长的结构掩码（注释与字符串内容置空、保留定界
符与换行），花括号/条目结构在掩码上判定，键值内容从原文对应区间读取。
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]  # noname-next/
UP = REPO.parent / "archive/upstream"
SOURCES = [(UP / "noname/apps/core/extension", 8), (UP / "noname-liuli/noname/extension", 29)]
SCOPES = {"player", "global", "target", "source"}
SKIP_FILES = {"extension_old.js", "extension_mess.js"}
NON_SKILL_KEYS = {"trigger", "filter", "ai", "content", "mark", "intro", "mod", "group",
                  "chooseControl", "init", "derivation", "onRemove", "onChooseToUse"}


def mask(src: str) -> str:
    """等长掩码：注释与字符串内容置空格（保留引号、换行、结构字符）。"""
    out = list(src)
    i, n = 0, len(src)
    mode = None  # None | 'line' | 'block' | quote char
    while i < n:
        c = src[i]
        two = src[i : i + 2]
        if mode is None:
            if two == "//":
                mode = "line"
                out[i] = out[i + 1] = " "
                i += 2
                continue
            if two == "/*":
                mode = "block"
                out[i] = out[i + 1] = " "
                i += 2
                continue
            if c in "'\"`":
                mode = c
                i += 1
                continue
            i += 1
            continue
        if mode == "line":
            if c == "\n":
                mode = None
            else:
                out[i] = " "
            i += 1
            continue
        if mode == "block":
            if two == "*/":
                out[i] = out[i + 1] = " "
                mode = None
                i += 2
                continue
            if c != "\n":
                out[i] = " "
            i += 1
            continue
        # 字符串内：保留定界符，内容置空
        if c == "\\" and i + 1 < n:
            out[i] = out[i + 1] = " "
            i += 2
            continue
        if c == mode:
            mode = None
        else:
            out[i] = " "
        i += 1
    return "".join(out)


def balanced(m: str, start: int) -> tuple[int, int]:
    """m[start] == '{'：返回（块内容区间 [s,e)，掩码坐标）。"""
    assert m[start] == "{"
    depth, i, n = 0, start, len(m)
    while i < n:
        c = m[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return (start + 1, i), i + 1
        i += 1
    raise ValueError("unbalanced braces")


def top_level_entries(m: str, src: str, lo: int, hi: int) -> dict[str, tuple[int, int]]:
    """对象字面量顶层条目：{name: 值区间}（区间为掩码=原文坐标）。"""
    out: dict[str, tuple[int, int]] = {}
    depth = 0
    i = lo
    while i < hi:
        c = m[i]
        if c in "{[(":
            depth += 1
        elif c in "}])":
            depth -= 1
        elif depth == 0:
            mkey = re.match(r"\s*([A-Za-z_$][\w$]*)\s*:", m[i:hi])
            if mkey and (i == lo or m[i - 1].isspace() or m[i - 1] in ",{"):
                name = mkey.group(1)
                j = i + mkey.end()
                vdepth, k = 0, j
                while k < hi:
                    ch = m[k]
                    if ch in "{[(":
                        vdepth += 1
                    elif ch in "}])":
                        if vdepth == 0:
                            break
                        vdepth -= 1
                    elif ch == "," and vdepth == 0:
                        break
                    k += 1
                out.setdefault(name, (j, k))
                i = k
                continue
        i += 1
    return out


def block_text(src: str, span: tuple[int, int]) -> str:
    return src[span[0] : span[1]].strip()


def events_of_trigger(m: str, src: str, lo: int, hi: int) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    for key, (a, b) in top_level_entries(m, src, lo, hi).items():
        value = src[a:b]
        if key in SCOPES:
            for ev in re.findall(r"['\"](\w+)['\"]", value):
                pairs.append((key, ev))
        elif re.fullmatch(r"\w+", key):
            pairs.append(("direct", key))
    return pairs


def parse_skill(name: str, m: str, src: str, lo: int, hi: int) -> dict:
    entries = top_level_entries(m, src, lo, hi)
    info: dict = {"name": name, "events": [], "priority": None, "silent": False,
                  "flags": [], "forced": False}
    tspan = entries.get("trigger")
    if tspan:
        a0 = tspan[0]
        while a0 < tspan[1] and m[a0] in " \t":
            a0 += 1
        if m[a0] == "{":
            (a, b), _ = balanced(m, a0)
            info["events"] = events_of_trigger(m, src, a, b)
    if "priority" in entries:
        mm = re.match(r"\s*(-?\d+(?:\.\d+)?)", block_text(src, entries["priority"]))
        if mm:
            info["priority"] = float(mm.group(1))
    info["silent"] = re.match(r"\s*true\b", block_text(src, entries.get("silent", (0, 0)))) is not None
    info["forced"] = re.match(r"\s*true\b", block_text(src, entries.get("forced", (0, 0)))) is not None
    for flag in ("equipSkill", "cardSkill", "ruleSkill"):
        if re.match(r"\s*true\b", block_text(src, entries.get(flag, (0, 0)))):
            info["flags"].append(flag)
    return info


def effective_priority(skill: dict) -> float:
    p = (skill["priority"] or 0) * 100
    if skill["silent"]:
        p += 1
    p -= 25 * ("equipSkill" in skill["flags"])
    p -= 50 * ("cardSkill" in skill["flags"])
    p -= 75 * ("ruleSkill" in skill["flags"])
    return p


def scan_source(root: Path) -> dict:
    skills: dict[str, dict] = {}
    translates: set[str] = set()
    generals: dict[str, list[str]] = {}
    files = [p for p in root.rglob("*.js") if p.name not in SKIP_FILES and "image" not in p.parts]
    for path in files:
        try:
            src = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        m = mask(src)
        for mt in re.finditer(r"\n\s*([A-Za-z_$][\w$]*)\s*:\s*\{", m):
            name = mt.group(1)
            if name in NON_SKILL_KEYS:
                continue
            try:
                (lo, hi), _ = balanced(m, m.index("{", mt.start() + len(mt.group(0)) - 1))
            except ValueError:
                continue
            if "trigger" in top_level_entries(m, src, lo, hi):
                if name not in skills:
                    skills[name] = parse_skill(name, m, src, lo, hi)
            elif "skills" in top_level_entries(m, src, lo, hi):
                if name not in generals and name not in ("character", "card"):
                    span = top_level_entries(m, src, lo, hi)["skills"]
                    lst = re.findall(r"['\"](\w+)['\"]", src[span[0] : span[1]])
                    if lst:
                        generals.setdefault(name, lst)
        # lib.skill.NAME = {...} / lib.skill["NAME"] = {...}
        for ms in re.finditer(r"lib\.skill(?:\[(['\"])(\w+)\1\]|\.(\w+))\s*=\s*\{", m):
            name = ms.group(2) or ms.group(3)
            if name and name not in skills:
                try:
                    (lo, hi), _ = balanced(m, m.index("{", ms.end() - 1))
                except ValueError:
                    continue
                if "trigger" in top_level_entries(m, src, lo, hi):
                    skills[name] = parse_skill(name, m, src, lo, hi)
        # 翻译：translate/translates 对象（`:` 或 `const x = {` 形式）顶层键 + lib.translate.NAME 赋值
        for mt in re.finditer(r"\b(?:translates|translate)\s*[:=]\s*\{", m):
            a0 = m.index("{", mt.end() - 1)
            try:
                (lo, hi), _ = balanced(m, a0)
            except ValueError:
                continue
            translates.update(top_level_entries(m, src, lo, hi).keys())
        for mt in re.finditer(r"lib\.translate(?:\[(['\"])(\w+)\1\]|\.(\w+))\s*=", m):
            translates.add(mt.group(2) or mt.group(3))
    return {"skills": skills, "translates": translates, "generals": generals,
            "files": len(files)}


def hits_of(source: str, data: dict) -> list[dict]:
    out: list[dict] = []
    skills, translates = data["skills"], data["translates"]
    for general, owned in data["generals"].items():
        by_event: dict[str, dict[float, dict]] = {}
        for sname in dict.fromkeys(owned):
            skill = skills.get(sname)
            if not skill or skill["silent"] or sname not in translates:
                continue
            p = effective_priority(skill)
            for scope, ev in skill["events"]:
                by_event.setdefault(ev, {}).setdefault(p, {})[sname] = scope
        for ev, groups in by_event.items():
            for p, members in groups.items():
                if len(members) > 1:
                    scopes = sorted(set(members.values()))
                    out.append({
                        "source": source, "general": general, "event": ev,
                        "priority": p if p % 1 else int(p),
                        "skills": sorted(members),
                        "scope_of": {k: v for k, v in sorted(members.items())},
                        "same_scope": len(scopes) == 1,
                        "any_forced": any(skills[k]["forced"] for k in members),
                    })
    return out


def main() -> None:
    report = {"sources": [], "totals": {}}
    all_hits: list[dict] = []
    for root, _ in SOURCES:
        for source in sorted(p for p in root.iterdir() if p.is_dir()):
            data = scan_source(source)
            hits = hits_of(source.name, data)
            all_hits.extend(hits)
            report["sources"].append({
                "source": source.name, "repo": root.parts[-3], "files": data["files"],
                "skills": len(data["skills"]),
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
    out_dir = Path(__file__).resolve().parent
    (out_dir / "hits.json").write_text(
        json.dumps({"report": report, "hits": all_hits}, ensure_ascii=False, indent=1),
        encoding="utf-8")
    for s in report["sources"]:
        print(f"{s['repo']}/{s['source']}: files={s['files']} skills={s['skills']}"
              f"(trigger={s['skills_with_trigger']}) generals={s['generals']} hit_groups={s['hit_groups']}")
    print(json.dumps(report["totals"], ensure_ascii=False, indent=1))


if __name__ == "__main__":
    sys.stdout.reconfigure(encoding="utf-8")
    main()
