#!/usr/bin/env python3
"""Enumerate every phase-pinned filter arm and classify it against upstream triggers.

The native host filters pin a specific `Phase` in many arms:

    EventData::Phase { player, phase: Phase::Judge, begin: true } if ...

Upstream, the same skill often uses `phaseBegin` (any phase) or `phaseEnd`
(turn end, after every phase).  Pinning the wrong phase is the single largest
family of rule differences found so far (G14: 10, G20: 5, plus one End-pinned
variant), so this script replaces ad-hoc sampling with an enumeration:

1. collect every arm containing `Phase::(Judge|Draw|Play|Discard|End), begin: <bool>`;
2. resolve behavior -> host key (`keys.rs`) -> skill name (package JSON);
3. resolve the upstream trigger (scan `archive/upstream` for the skill name,
   trying the forms that have historically hidden definitions: bare name,
   `lib.skill.X =`, leading/trailing underscores, extension prefix on/off);
4. classify:

   * `phaseBegin` + pinned phase          -> G14 family (should not pin)
   * `phaseEnd`   + `begin: true`         -> G20 (one step early)
   * `phaseXxxBegin` matching the pin     -> OK
   * `phaseXxxEnd` + `begin: false`       -> OK
   * everything else                      -> REVIEW

Usage:
    python3 -B scripts/report_phase_arms.py             # TSV + summary
    python3 -B scripts/report_phase_arms.py --check     # exit 1 on new REVIEW rows
"""

from __future__ import annotations

import argparse
import glob
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates" / "engine-native" / "src" / "content" / "host"
UPSTREAM = ROOT.parent / "archive" / "upstream"

ARM_RE = re.compile(r"Phase::(Judge|Draw|Play|Discard|End),\s*begin:\s*(true|false)")
#: Some arms pin the phase by comparison instead of destructuring:
#:   `Phase { player, phase, begin } if *player == owner && *phase == Phase::End && *begin`
#: Both shapes are real, so both are enumerated (missing the second produced
#: false "unpinned" verdicts in an earlier round).
EQ_RE = re.compile(r"\*phase\s*==\s*crate::event::Phase::(Judge|Draw|Play|Discard|End)")
NEG_BEGIN_RE = re.compile(r"!\*begin\b")
BEH_RE = re.compile(r"HostBehavior::([A-Za-z0-9_]+)\s*=>")

#: Hand-checked upstream triggers for skills the automatic scan cannot resolve.
MANUAL_TRIGGERS = {
    "diyguhuo": 'player: "phaseBegin"',
    "gujian_food_yougeng": 'player: "phaseBegin"',
    "huanle_xuelunyang": 'player: "phaseBegin"',
    "zhenjiu2": 'player: "phaseBegin"',
    "lexue": 'player: "phaseBegin"',
    "fengqi": "player: ['phaseZhunbeiBegin','phaseJieshuBegin']",
    "tanhuadong": 'player: "phaseJieshuBegin"',
    "phase_end_recover_or_draw": 'player: "phaseEnd"',
    "gujian_food_liyutang": 'player: "phaseEnd"',
    "gujian_food_mapodoufu": 'player: "phaseJieshuBegin"',
    "gujian_food_ziyangdan": 'player: "phaseBegin"',
    "gujian_food_xiajiao": 'player: "phaseDrawBegin"',
    # 装备/机巧卡的效果挂在卡片的 `global: 'g_<name>'` 技能上 —— 第 7 种「查不到」形态。
    "qinglongzhigui": 'player: "phaseBegin"',
    "huanglinzhicong": 'player: "phaseBegin"',
    # 复合技（`group: ['miedao1','miedao2']`）：父技能无 trigger，真实触发在子技能上
    "miedao": "player:'phaseDrawBegin' | player:'phaseDiscardBegin'",
    # 化金：主触发是 huajin2 的 `source:'damageBegin'`；原生的 `Phase::Judge, begin: true`
    # 分支对应子技能 huajin3 的 `player:'phaseBegin'`（回合开始移除效果）——
    # 判定阶段开始与回合开始几乎同刻，按**近似等价**处理，不计缺陷（详见规则基准）。
    "huajin2": "source:'damageBegin'",
}

#: Engine phase -> the upstream event name stem (`Play` is `phaseUse…` in noname).
PHASE_STEM = {
    "Judge": "Judge",
    "Draw": "Draw",
    "Play": "Use",
    "Discard": "Discard",
    "End": "Jieshu",
}

#: Behavior-level overrides for internal sub-behaviors that have **no `host:` key**
#: (so neither the key nor the package skill name can find the upstream skill).
BEHAVIOR_TRIGGERS = {
    "SwdQinglongzhiguiHeld": 'player: "phaseBegin"',      # g_qinglongzhigui（卡片 global 技能）
    "SwdHuanglinzhicongHeld": 'player: "phaseBegin"',     # g_huanglinzhicong
    # 三条 gwent 天气状态技：定义在 杀海拾遗/card/gwent/skill.js（名 gw_*，非 gw_*_state）
    "GwentBirinState": 'player: "phaseUseAfter"',
    "GwentCiguState": 'player: "phaseDrawBegin"',
    "GwentQinpenState": 'player: "phaseDiscardAfter"',
    # 两条相邻相位（弃牌阶段结束 vs 结束阶段开始）
    "SwdWangchenHeld": 'player: "phaseDiscardEnd"',
    "SwdXielv": 'player: "phaseDiscardEnd"',
}

#: Rows whose verdict is a documented semantic choice rather than a defect.
SEMANTIC_CHOICE = {
    "GujianFoodTanhuadong": "上游代码 phaseEnd 与文案『剩余N回合』矛盾，原生取每回合一次（见规则基准）",
}


def host_keys() -> dict[str, str]:
    src = (HOST / "keys.rs").read_text(encoding="utf-8")
    return {m.group(1): m.group(2) for m in re.finditer(r'HostBehavior::([A-Za-z0-9_]+)\s*=>\s*"(host:[^"]+)"', src)}


def skill_names() -> dict[str, str]:
    """host key -> package skill name."""
    out: dict[str, str] = {}
    for path in glob.glob(str(ROOT / "packages" / "extensions" / "*" / "assets" / "*.json")):
        try:
            doc = json.load(open(path, encoding="utf-8"))
        except Exception:
            continue
        for skill in doc.get("skills") or []:
            if isinstance(skill, dict) and skill.get("mapping"):
                out[skill["mapping"]] = skill.get("name")
    return out


_UPSTREAM_CACHE: dict[str, str] | None = None


def upstream_sources() -> dict[str, str]:
    global _UPSTREAM_CACHE
    if _UPSTREAM_CACHE is None:
        _UPSTREAM_CACHE = {}
        for path in glob.glob(str(UPSTREAM / "**" / "*.js"), recursive=True):
            try:
                _UPSTREAM_CACHE[path] = Path(path).read_text(encoding="utf-8", errors="replace")
            except Exception:
                continue
    return _UPSTREAM_CACHE


def _block(text: str, start: int) -> str:
    i = start - 1
    depth = 0
    while i < len(text):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[start - 1 : i + 1]
        i += 1
    return text[start - 1 :]


def upstream_trigger(skill: str, key: str = "") -> str:
    """Resolve the upstream trigger for a behavior.

    `MANUAL_TRIGGERS` is keyed by **host key** (the `host:` prefix stripped),
    matching `report_g14.py`; skill names are the fallback.
    """
    if key in MANUAL_TRIGGERS:
        return MANUAL_TRIGGERS[key]
    candidates = [name for name in (skill, key, key.rsplit("_", 1)[-1]) if name]
    for candidate in candidates:
        found = _lookup_trigger(candidate)
        if found:
            return found
    return ""


def package_path_for_key(key: str) -> str:
    """`host:` 键所属包的 `upstream_path` —— 用于消解「同名技能跨包」的歧义。"""
    if not key:
        return ""
    for path in glob.glob(str(ROOT / "packages" / "extensions" / "*" / "assets" / "*.json")):
        try:
            doc = json.loads(Path(path).read_text(encoding="utf-8"))
        except Exception:
            continue
        for skill in doc.get("skills") or []:
            if isinstance(skill, dict) and skill.get("mapping") == (key if key.startswith("host:") else f"host:{key}"):
                return (doc.get("upstream_path") or "").strip("/")
    return ""


def upstream_trigger_with_path(skill: str, key: str = "") -> tuple[str, str]:
    """(trigger, 上游文件路径) —— 路径用于把缺陷归到「代码实际所在来源」。"""
    if key in MANUAL_TRIGGERS:
        return MANUAL_TRIGGERS[key], ""
    # 只试「包内技能名」与「host 键本身」；**不再用键尾段**做回退
    #（`shengke_check` 曾因此解析到另一扩展的同名技能，得出错误 trigger）
    # 第 7 种名字形态：卡牌的 `global: 'g_<name>'` 技能（如 `baishouzhihu` → `g_baishouzhihu`）
    candidates = [name for name in (skill, key, f"g_{key}" if key else "", f"g_{skill}" if skill else "") if name]
    prefer = package_path_for_key(key)
    for candidate in candidates:
        found = _lookup_trigger_with_path(candidate, prefer)
        if found[0]:
            return found
    return "", ""



def _group_child_triggers(block: str) -> str:
    """`group: ['azhu2']` 的子技能触发面也要算进来。

    例：`huajin2` 是 `trigger:{source:'damageBegin'}` 且 `group:'huajin3'`，
    而 `huajin3` 是 `trigger:{player:'phaseBegin'}`（回合结束移除效果）。
    只读父技能会漏掉 phaseBegin，从而把原生的 Phase 分支误判成「自加分支」。
    """
    group = re.search(r"group\s*:\s*(?:\[([^\]]*)\]|['\"]([A-Za-z0-9_]+)['\"])", block)
    if not group:
        return ""
    raw = group.group(1) or group.group(2) or ""
    children = re.findall(r"['\"]([A-Za-z0-9_]+)['\"]", raw) or [raw]
    extra = []
    for child in children:
        child_trigger, _path = _lookup_trigger_with_path(child)
        if child_trigger and not child_trigger.startswith("("):
            extra.append(child_trigger)
    return (" | " + " | ".join(extra)) if extra else ""


def _lookup_trigger_with_path(skill: str, prefer: str = "") -> tuple[str, str]:
    if not skill:
        return "", ""
    patterns = [
        r"\n\s*" + re.escape(skill) + r"\s*:\s*\{",
        r"lib\.skill\." + re.escape(skill) + r"\s*=\s*\{",
        r"\n\s*_" + re.escape(skill) + r"_\s*:\s*\{",
        r"lib\.skill\._" + re.escape(skill) + r"_\s*=\s*\{",
    ]
    items = list(upstream_sources().items())
    if prefer:
        # 优先在「该键所属包」的 upstream_path 下查找（消解同名技能跨包歧义）
        items.sort(key=lambda kv: 0 if prefer in kv[0].replace("\\", "/") else 1)
    for path, text in items:
        for pattern in patterns:
            m = re.search(pattern, text)
            if not m:
                continue
            block = _block(text, m.end())
            head = block[:60]
            if re.search(r"\btype\s*:", head) and "trigger" not in head:
                continue
            trigger = re.search(r"trigger\s*:\s*\{([^}]*)\}", block)
            if trigger:
                return " ".join(trigger.group(1).split()), path
            if "enable" in block:
                return "(enable 主动技)", path
            return "(无 trigger)", path
    return "", ""


def _lookup_trigger(skill: str) -> str:
    if not skill:
        return ""
    patterns = [
        r"\n\s*" + re.escape(skill) + r"\s*:\s*\{",
        r"lib\.skill\." + re.escape(skill) + r"\s*=\s*\{",
        r"\n\s*_" + re.escape(skill) + r"_\s*:\s*\{",
        r"lib\.skill\._" + re.escape(skill) + r"_\s*=\s*\{",
    ]
    for text in upstream_sources().values():
        for pattern in patterns:
            m = re.search(pattern, text)
            if not m:
                continue
            block = _block(text, m.end())
            # 跳过卡片定义（`type:...` 开头），继续找同名技能定义。
            head = block[:60]
            if re.search(r"\btype\s*:", head) and "trigger" not in head:
                continue
            trigger = re.search(r"trigger\s*:\s*\{([^}]*)\}", block)
            if trigger:
                return " ".join(trigger.group(1).split())
            if "enable" in block:
                return "(enable 主动技)"
            return "(无 trigger)"
    return ""


def arms() -> list[tuple[str, str, str, str, int]]:
    """[(behavior, phase, begin, file, line)]"""
    rows: list[tuple[str, str, str, str, int]] = []
    for path in sorted(HOST.rglob("*.rs")):
        if "filtering" not in str(path):
            continue
        text = path.read_text(encoding="utf-8")
        lines = text.splitlines()
        for idx, line in enumerate(lines):
            behavior = None
            for back in range(idx, max(0, idx - 8), -1):
                hit = BEH_RE.search(lines[back])
                if hit:
                    behavior = hit.group(1)
                    break
            for m in ARM_RE.finditer(line):
                rows.append((behavior or "?", m.group(1), m.group(2), path.name, idx + 1))
                continue
            hit = EQ_RE.search(line)
            if hit:
                # `!*begin` in the same line/arm means the end edge.
                begin = "false" if NEG_BEGIN_RE.search(line) else "true"
                rows.append((behavior or "?", hit.group(1), begin, path.name, idx + 1))
    return rows


def _behavior_name_candidates(behavior: str) -> list[str]:
    """从 `HostBehavior` 变体名推上游技能名（**仅当没有 `host:` 键时**使用）。

    卡片全局技能没有 host 键，变体名形如 `SwdBaishouzhihuHeld` ⇒ 上游是
    `g_baishouzhihu`（卡牌的 `global:` 技能）。这里只做**去前缀/去后缀**，
    再试 `g_` 与原名两种形态 —— 不做任意切词猜测。
    """
    name = behavior
    for suffix in ("Held", "Card", "Use", "Gain", "Remain"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
    for prefix in ("Swd", "Wudian", "Xianjian", "Gujian", "Yunchou", "Hearth", "Gwent", "Mtg", "Shahai", "Ow"):
        if name.startswith(prefix) and len(name) > len(prefix):
            name = name[len(prefix):]
            break
    snake = re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()
    return [f"g_{snake}", f"g_{behavior.lower()}", snake, behavior.lower()]


def resolve_trigger(behavior: str, key: str, skill: str) -> str:
    """Upstream trigger for one arm: behavior override -> key -> skill -> 变体名推断。"""
    if behavior in BEHAVIOR_TRIGGERS:
        return BEHAVIOR_TRIGGERS[behavior]
    found = upstream_trigger(skill, key.removeprefix("host:"))
    if found or key:
        return found
    for candidate in _behavior_name_candidates(behavior):
        hit = upstream_trigger(candidate, candidate)
        if hit:
            return hit
    return ""



def behavior_has_damage_and_phase_two_faces(behavior: str) -> bool:
    """该行为的「伤害面 + 相位面」是否都有上游依据。

    目前只登记了**已逐条读过上游**的：`SwdHuajin2`（huajin2 的 `source:'damageBegin'`
    + 子技能 huajin3 的 `player:'phaseBegin'`）。
    """
    return behavior in {"SwdHuajin2"}


def classify(behavior: str, phase: str, begin: str, trigger: str) -> str:
    if behavior in SEMANTIC_CHOICE:
        return "语义选择"
    if not trigger:
        return "REVIEW(上游未定位)"
    if trigger.startswith("(enable"):
        # 上游是主动技（无 trigger），原生却挂了阶段触发 ⇒ 凭空新增
        return "凭空新增触发"
    if "phaseBegin" in trigger and "phaseZhunbei" not in trigger:
        # 上游是「回合开始」。原生**没有准备阶段**（`Phase` 只有 Judge/Draw/Play/Discard/End，
        # **Judge 是第一相位**）⇒ 钉在 `Judge, begin: true` 等价于回合开始，不计缺陷；
        # 钉在其它阶段才是真错位。（第 804 轮据此撤回 13 行，仅留 `GujianFoodZiyangdan`）
        if PHASE_STEM[phase] == "Judge" and begin == "true":
            return "OK"
        return "G14(钉死阶段)"
    stem = PHASE_STEM[phase]
    if f"phase{stem}Begin" in trigger and begin == "true" and PHASE_STEM[phase] == "Judge":
        # 上游是「回合开始」（`phaseBegin`），原生钉在 `Judge, begin: true`。
        # 原生没有「准备阶段」（`Phase` 只有 Judge/Draw/Play/Discard/End，**Judge 是第一相位**）
        # ⇒ **判定阶段开始 == 回合开始** ⇒ **等价**，不计缺陷。
        # （第 804 轮：13 行据此撤回；只有钉在 End/Play/Draw/Discard 的才是真错位）
        return "OK"
    if f"phase{stem}Begin" in trigger and begin == "false":
        # 上游在阶段「开始」生效，原生却在「结束」⇒ 晚一步（摸牌数已定，效果失效）
        return "G22(晚一步)"
    if f"phase{stem}Begin" in trigger:
        return "OK" if begin == "true" else "REVIEW(begin 相反)"
    if f"phase{stem}End" in trigger:
        return "OK" if begin == "false" else "REVIEW(begin 相反)"
    if f"phase{stem}After" in trigger or f"phase{stem}End" in trigger:
        # 「阶段之后/结束」与原生 `begin: false` 是同一帧
        return "OK" if begin == "false" else "REVIEW(begin 相反)"
    if f"phase{stem}Before" in trigger:
        return "G15(之前档)" if begin == "true" else "REVIEW(begin 相反)"
    if "phaseAfter" in trigger:
        return "G15(phaseAfter)" if begin == "false" else "REVIEW(begin 相反)"
    if "phaseBegin" in trigger and "phaseZhunbei" not in trigger:
        return "G14(钉死阶段)"
    if "phaseEnd" in trigger:
        return "G20(早一步)" if begin == "true" else "OK"
    if "phaseZhunbeiBegin" in trigger:
        # 上游是「准备阶段开始」。原生**没有准备阶段**，`Judge, begin: true` 即回合开始 ⇒
        # 与 prepare 开始同刻（prepare 是回合的第一个阶段）⇒ **等价**；钉在其它相位才是错位。
        # （第 805 轮：2 行据此撤回，G18 类消失）
        return "OK" if PHASE_STEM[phase] == "Judge" and begin == "true" else "G18(准备阶段缺档)"
    if "phaseDiscardEnd" in trigger and phase == "End" and begin == "true":
        # 弃牌阶段结束 → 结束阶段开始 是相邻帧，差异仅在「结束阶段自身动作之前/之后」
        return "相邻相位(可接受)"
    if "damageBegin" in trigger:
        # 上游的主动面是「伤害前」；本行为里另有一条伤害分支是**对应上游的**，
        # 阶段分支才是「另一面」。若该行为的函数体注释说明分支用途（如 huajin2 的
        # 另一面是「移除效果」），按**两面对应**处理 ⇒ OK；否则记为自加分支。
        if behavior_has_damage_and_phase_two_faces(behavior):
            return "OK"
        return "自加分支(上游伤害前)"
    return "REVIEW(非阶段触发)"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit 1 when a REVIEW row appears")
    args = ap.parse_args()

    keys = host_keys()
    skills = skill_names()
    out = []
    counts: dict[str, int] = {}
    for behavior, phase, begin, file, line in arms():
        key = keys.get(behavior, "")
        skill = skills.get(key, "") or ""
        trigger = resolve_trigger(behavior, key, skill)
        verdict = classify(behavior, phase, begin, trigger)
        counts[verdict] = counts.get(verdict, 0) + 1
        out.append((behavior, phase, begin, f"{file}:{line}", skill, trigger, verdict))

    print("behavior\tphase\tbegin\t位置\tskill\t上游trigger\t判定")
    for row in sorted(out, key=lambda r: (r[6], r[0])):
        print("\t".join(row))
    print(f"\n共 {len(out)} 个相位钉死臂", file=sys.stderr)
    for verdict, n in sorted(counts.items(), key=lambda kv: -kv[1]):
        print(f"  {verdict}: {n}", file=sys.stderr)

    review = [r for r in out if r[6].startswith("REVIEW")]
    if review:
        print(f"\n{len(review)} 行需人工复核", file=sys.stderr)
        return 1 if args.check else 0
    print("\n无 REVIEW 行", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
