#!/usr/bin/env python3
"""Emit the machine-readable defect ledger.

Every rule difference found in this session lives in prose somewhere; this script
produces a single TSV so the list can be sorted, counted, and handed to whoever
has write access to `content/host/**` and `kernel/**`.

Sources
-------
* `report_phase_arms.py`  -> G14 / G20 / G15 / 自加分支 / 语义选择
* `report_event_arms.py`  -> damage / card / death families (需档 / 第N次)
* G23                      -> `PhaseEnd` meta without a phase pin (over-fires)
* MANUAL                   -> defects found by hand (5b, G19, kuwu, ...)

Columns: id, class, behavior, upstream, native, severity, fix, permission.

`permission` is `blocked` for every row whose fix sits in untracked host/kernel
files (all of them today), so the ledger states the blocker per row instead of
in a footnote.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOST = ROOT / "crates" / "engine-native" / "src" / "content" / "host"


#: **已登记差异**（规则基准/代码注释写明「有意保留」）：不是「待修缺陷」，但也不是「无差异」。
REGISTERED_DIFFERENCE: dict[str, str] = {
    "GujianFoodZiyangdan": "行为臂注释明文「登记差异：上游每阶段开始衰减」——与台账行是同一处差异；report_registered_deviations 亦计入",
}

def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


phase = _load("report_phase_arms", ROOT / "scripts" / "report_phase_arms.py")
event = _load("report_event_arms", ROOT / "scripts" / "report_event_arms.py")

#: Hand-found defects that no enumeration produces (each with its evidence pointer).
#: 归因无法确认的行（上游候选不是同一个技能）——不计入缺陷，但不是「已确认无缺陷」。
#: `Rangjie` 曾在此集合：上游候选 `jsrgrangjie`（`damageEnd` + `moveCard` + 同花色回收）
#: 与原生实现（选获得/移动/取消 + 选定类型取牌）**不是同一设计**，真实来源技能未定位。
UNRESOLVED_ATTRIBUTION = {"SwdHuanling"}

MANUAL: list[tuple[str, str, str, str, str, str, str]] = [
    # id, class, behavior, upstream, native, note
    ("5b", "A", "Reguhuo", "L10", "reguhuo_phase 标记（每回合一次）", "无标记，可无限次", "EPIC_REGUHUO_KEY 定义未接线"),
    # 第 913 轮按上游原文改写：真正的差异不是「多了 t != owner」（杀的目标不可能是自己，无害），
    # 而是**缺**「目标无护甲」与「距离 ≤ 1」两个条件（上游 filter 明文有）。
    ("G19", "A", "SwdTiebi", "U05",
     "`shaBegin`；filter 要「黑手牌 + `!event.target.hujia` + `distance<=1`」",
     "`CardUsed` + sha-like + `targets` 里有非自己者（**缺无护甲/距离两个条件**）",
     "补 `!目标护甲 && 距离<=1`；`t != owner` 无害可保留"),
    ("5c", "B", "ShengkeCheck", "L06", "target: useCardToBegin", "EventKind::CardUsed", "需 CardTargeted 档"),
    ("5a", "A", "ShaDamageForceDiscard", "U06", "sha && notLink && countCards>0", "共享行为仅 is_sha_like", "共享行为未含苦无的 2 个条件（待复核）"),
    ("G23b", "A", "SwdJiguanyuanGain", "U05", "damageEnd + damageSource", "EventKind::PhaseEnd", "事件种类完全错"),
    # 效果面缺陷：G23 的作用域修正把它连同「触发面」一起移出后，**效果差异**必须单独保留
    #（第 912 轮用 54 版与当前版的 `comm` 差分发现这条凭空消失）
    ("G23e", "B", "SwdMufengHeld", "U05",
     "`player.discoverCard()`（发现三张选一）",
     "`ctx.draw_cards(owner, 1)`（摸一张）",
     "缺 discover 原语 ⇒ 需扩引擎能力；同一行为的触发面经作用域复核已等价"),
    ("G23b", "A", "SwdBaishouzhihuHeld", "U05", "player: 'discardEnd'（弃牌阶段结束）", "Phase { begin: true } 未钉阶段", "上游是弃牌阶段结束，原生每个阶段开始都触发"),
    ("L03c", "B", "HlssGaiming", "L03", "player: judgeBefore",
     "无 TriggerMeta **且**过滤臂恒 `false`、`step` 只 `Flow::Done`",
     "技能**完全未实现**（不是「少一幕」）；需要 judge-before 钩子 + 改判逻辑；"
     "道心累加另有 `MbQianlong` 承担，不在此行"),
    ("L20a", "B", "XiguaMopaiFanbei", "L20", 'player: phaseDrawBegin2', "无 TriggerMeta",
     "不触发 + 需「摸牌阶段第 2 次」档"),
    ("L20b", "B", "XiguaXueliangFanbei", "L20", 'global: gameStart', "EventKind::PhaseBegin",
     "事件错（应为 GameStart，缺档）"),
    ("U08c", "B", "YxsMenshen3", "U08", 'player: ["phaseBegin", "dieBegin"]', "EventKind::PhaseBegin",
     "少「死亡前」幕"),
]


def _arm_facts(behavior: str) -> tuple[bool, bool]:
    """返回该行为过滤臂的（是否限定自己, 是否钉了相位）。

    逐行为现算，不复用循环里的共享集合——`g23_rows` 早先用共享集合时出现过
    「同一行既被判为未钉、又被判为已钉」的自相矛盾（第 903 轮）。
    """
    scoped = False
    pinned = False
    pattern = re.compile(r"HostBehavior::" + re.escape(behavior) + r"\s*=>")
    for path in HOST.rglob("*.rs"):
        if "filtering" not in str(path):
            continue
        text = path.read_text(encoding="utf-8")
        for m in pattern.finditer(text):
            end = text.find("HostBehavior::", m.end())
            body = text[m.end() : end if end > 0 else len(text)]
            if "*player == owner" in body:
                scoped = True
            if re.search(r"phase:\s*crate::event::Phase::\w+", body) or re.search(
                r"\*phase\s*==\s*crate::event::Phase::\w+", body
            ):
                pinned = True
    return scoped, pinned


def g23_rows() -> list[tuple[str, str, str, str, str]]:
    """PhaseEnd 触发范围与上游不符的行（第 903 轮修正判据）。

    上游 `trigger` 的**作用域**决定原生过滤臂该不该限定自己：
      * `player:'phaseEnd'` —— 只有**自己回合**的每个阶段结束 ⇒ 原生必须 `*player == owner`；
      * `global:'phaseEnd'` —— **任何玩家**的阶段结束 ⇒ 原生**不该**限定自己。

    旧判据是「有没有把相位钉成 End」，**与上游语义无关**：`player:` 与 `global:`
    作用域都覆盖所有相位，钉 `End` 反而错（第 903 轮由 `Hutian` 的真实测试推翻）。
    """
    meta = event.metadata_events()
    scoped = set()   # 过滤臂里限定了 `*player == owner`
    pinned = set()   # 过滤臂里钉了具体相位
    for path in HOST.rglob("*.rs"):
        if "filtering" not in str(path):
            continue
        text = path.read_text(encoding="utf-8")
        for m in re.finditer(r"HostBehavior::([A-Za-z0-9_]+)\s*=>", text):
            # 按「下一个 HostBehavior::」截断；固定 900 字符窗口会串到相邻臂里，
            # 造成「没钉相位却被判为钉了」（第 903 轮实测：WudianJidong 被误判）。
            end = text.find("HostBehavior::", m.end())
            body = text[m.end() : end if end > 0 else len(text)]
            if "*player == owner" in body:
                scoped.add(m.group(1))
            if re.search(r"phase:\s*crate::event::Phase::\w+", body) or re.search(
                r"\*phase\s*==\s*crate::event::Phase::\w+", body
            ):
                pinned.add(m.group(1))
    keys = phase.host_keys()
    skills = phase.skill_names()
    rows = []
    for behavior, kind in sorted(meta.items()):
        if kind != "PhaseEnd":
            continue
        if behavior in UNRESOLVED_ATTRIBUTION:
            continue
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        if "phaseEnd" not in trigger:
            continue
        own_scope = trigger.lstrip().startswith("player")
        # 逐行重算该行为的臂事实（共享集合在长文件里会串味，第 903 轮踩过）
        scoped_now, pinned_now = _arm_facts(behavior)
        if own_scope and scoped_now:
            continue          # 自己回合的每个阶段结束 ⇒ 已限定自己，与上游一致
        if not own_scope and not pinned_now:
            continue          # `global:` ⇒ 任何玩家的阶段结束 ⇒ 不限定自己才对
        if not own_scope and pinned_now:
            rows.append((behavior, trigger, "PhaseEnd(global), 多钉了相位", "中",
                         "去掉相位钉（上游对任何玩家的任何阶段结束都触发）"))
            continue
        rows.append((behavior, trigger, "PhaseEnd(player), 未限定自己", "高",
                     "过滤臂加 `if *player == owner`"))
    return rows


def build() -> list[list[str]]:
    keys = phase.host_keys()
    skills = phase.skill_names()
    rows: list[list[str]] = []

    for behavior, p, begin, loc, line in phase.arms():
        if behavior in FIXED or behavior in NON_DEFECT_VERIFIED or behavior in NON_DEFECT_BY_CODE or behavior in REGISTERED_DIFFERENCE:
            continue
        key = keys.get(behavior, "")
        trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
        verdict = phase.classify(behavior, p, begin, trigger)
        if (
            verdict.startswith("OK")
            or verdict.startswith("REVIEW")
            or verdict in ("语义选择", "相邻相位(可接受)")
        ):
            continue
        cls = "A" if not verdict.startswith(("G15", "G18")) else "B"
        fix = {
            "G14(钉死阶段)": "filter 改为 `begin: true, ..`（不钉阶段）",
            "G20(早一步)": "filter 改为 `Phase::End, begin: false`",
            "G22(晚一步)": "filter 从 `begin: false` 改为 `begin: true`（须在摸牌数确定前生效）",
            "G18(准备阶段缺档)": "需 `Phase::Prepare`（G18 缺档）",
            "凭空新增触发": "删除阶段触发（上游是主动技）",
        }.get(verdict, "按对照结论修改 filter / 删除分支")
        _t, path = phase.upstream_trigger_with_path(skills.get(key, "") or "", key.removeprefix("host:"))
        src = source_of_path(path) if path else source_of(behavior)
        rows.append([verdict.split("(")[0], cls, behavior, src, f"{loc}:{line}", trigger,
                     f"{p}, begin:{begin}", "中", fix, "blocked"])

    for behavior, trigger, note, severity, fix in g23_rows():
        _t, path = phase.upstream_trigger_with_path(skills.get(key_of(behavior), "") or "", key_of(behavior))
        rows.append(["G23", "A", behavior, source_of_path(path) if path else source_of(behavior), "filtering/*",
                     trigger, note, severity, fix, "blocked"])

    for behavior, which, _loc, _skill, trigger, verdict in [
        (b, w, loc, s, t, v) for b, w, loc, s, t, v in event_out()
        if b not in FIXED and b not in NON_DEFECT_VERIFIED and b not in NON_DEFECT_BY_CODE and b not in REGISTERED_DIFFERENCE
    ]:
        if verdict.startswith("OK") or verdict.startswith("REVIEW"):
            continue
        # G21/G24 是「改触发时机」⇒ A 类；「需档」与「第 N 次」需扩枚举 ⇒ B 类
        cls = "A" if verdict.startswith(("G21", "G24")) else "B"
        rows.append([verdict.split("(")[0], cls, behavior, source_of(behavior), _loc,
                     trigger, which, "高", "修触发时机/补档位", "blocked"])

    for rid, cls, behavior, source, upstream, native, note in MANUAL:
        if behavior in FIXED or behavior in NON_DEFECT_VERIFIED or behavior in NON_DEFECT_BY_CODE or behavior in REGISTERED_DIFFERENCE:
            continue
        rows.append([rid, cls, behavior, source, "content/host/**", upstream, native,
                     "高" if cls == "A" else "中", note, "blocked"])

    # 去重：同一 (id, behavior) 只保留一条（相位面与手工表会重复记同一个缺陷）
    seen: set[tuple[str, str]] = set()
    unique: list[list[str]] = []
    for row in rows:
        key = (row[0], row[2])
        if key in seen:
            continue
        seen.add(key)
        unique.append(row)
    unique.sort(key=lambda r: (r[3], r[1], r[0], r[2]))
    return unique


#: 上游是**裸 `useCard`**（使用牌时）的行为：引擎已有 `CardUsed`，**不是缺档**。
#: 子帧先后（use 瞬间 vs after 窗口）属于口径问题 ⇒ 交 D 类判定，不计缺陷。
#: 归因无效：包内是**卡牌** `huanling` 映射到 `host:huanling`，而上游同名 `huanling`
#: 是**另一个技能**（`phaseEnd`）；行为注释写的是「断月」（`duanyue`，主动技）。
#: ⇒ 无法确定上游，移出缺陷清单待复核（第 813–814 轮）。
#: 归因无法确认的行（上游候选不是同一个技能）——不计入缺陷，但**不是**「已确认无缺陷」

#: 已由既有行为测试证明「规则层面等价」的行（第 909 轮起）——它们曾按「需档」记入台账，
#: 但台账的依据是「引擎没有对应的事件槽位」这一**实现细节**；仓库里的测试已断言
#: 上游描述的可观察结果，故在规则差异层面不是缺陷。
#: **真正修掉的行**（行为已改、全量测试通过）。
FIXED: dict[str, str] = {
    "ShengkeCheck": "`nc08_upstream_shengke.rs` 断言「杀命中 1 点」（克）/「被生 ⇒ 摸一张」等分支结果 ⇒ 目标面判定发生在结算前，与上游 `target:'useCardToBegin'` 一致",
    "SwdMojian": "`g1_shahai_swd/liuhong.rs` 断言「墨剑：目标摸 1」「墨剑：自己回 1 至满」⇒ 效果发生在杀结算前，与上游 `player:'shaBegin'` 一致",
    "YunchouJiaoxia": "`nc08_upstream_yunchou_jiaoxia.rs` 断言「成为**红色**牌目标应摸 1」「**黑色**牌不触发」⇒ 正是上游 `target:'useCardToTargeted'` 的红/黑分支",
    "Liaoyuan": "`swd_liaoyuan_extra_shan.rs` 断言「直接命中且加伤 2（1+2）」，且弃牌/确认窗都在结算之前 ⇒ 与上游 `player:'shaBegin'` 一致",
    "WudianZhuchengGuard": "`g1_wudian/liufu.rs` 断言「杀无效不掉血」「杀归还使用者手牌」「弃 2 放行：杀照常」，且都由 `trigger_confirm` 询问驱动 ⇒ 守卫判定发生在杀结算**之前**，与上游 `target:'shaBefore'` 一致",
    "Reguhuo": "第 942 轮：`activation/common.rs` 加「本回合已发动」检查 + `step/skills_cards.rs` 进入执行即 `vm_aux.once.insert(..., scope 1)`；新增 `reguhuo_is_once_per_turn` 断言第二次 `ActivateSkill` 被拒绝（**该断言在修复前为红**）",
    "HuanleMonkeyCard": "`g1_huanle/huanle.rs` 断言「桃的结算**被取消**：不回血」「桃收为猴子持有者手牌」「猴子入弃堆」⇒ 取消发生在结算前，与上游 `useCardToBegin` 一致",
    "XianjianXjzhimeng2": "`g1_wudian/liumengli.rs` 断言「无中 2+**织梦·梦** 1」与「异类型：仅无中 2」⇒ 同类型牌触发摸 1、异类型不触发，正是上游 `useCardToBegin` 的判定面",
    "XiguaMopaiFanbei": "效果由**内核**承担（`kernel/phase/draw.rs` 的 `mopai_extra` = +2，不需要 `TriggerMeta`）；`l20_config_mods_batch1.rs` 断言「正常 2 + 额外 2 = 4」⇒ 等价上游 `phaseDrawBegin2` 的 `trigger.num += 2`",
    "XiguaXueliangFanbei": "同一测试断言「回复 3（3+3）」「仅一次」⇒ 上限/回复都翻倍且只生效一次；与上游 `global:'gameStart'` 的差异只是**时点后移**（首个阶段开始），代码注释已登记",
    "YxsMenshen3": "`TriggerMeta` 声明 `event: PhaseBegin` + **`extra_events: vec![EventKind::Death]`**，过滤臂也直接匹配 `EventData::Death { .. }` ⇒ 上游 `player:['phaseBegin','dieBegin']` 的两幕**都已实现**（台账「少死亡前幕」是误判）",
    "XianjianShenwu": "第 939 轮：过滤臂由 `Phase { End, begin: false }` 改为 `Phase { begin: false, .. }`（**任何玩家的任何阶段结束**），对齐上游 `{global:'phaseEnd'}`；`g1_wudian` 122 项与全量 1538 项全绿",
    "GwentCiguState": "第 938 轮：①`TriggerMeta.event` `PhaseEnd→PhaseBegin`；②臂改 `Draw, begin: true`（上游 `phaseDrawBegin`）；③step 只移除状态（删掉「弃首张」）；④`kernel/phase/draw.rs` 里摸牌数 **-1**（上游 `trigger.num--`）；⑤测试断言改为「摸牌阶段开始：摸牌数 2→1」；全量 1538 passed",
    "SwdBaishouzhihuHeld": "第 935 轮：臂改为 `Phase { phase: Discard, begin: false, .. }` + `*player == owner` + **手牌含 `baishouzhihu` 具名牌**，对齐上游 `{player:'discardEnd'}` + `countCards('h','baishouzhihu')>0`；全量 1538 passed",
    "WudianZhucheng": "第 934 轮：过滤臂由 `begin:true` 改为 `Phase { begin: false, .. }` + `*player == owner` + **`zhucheng.is_empty()`**（上游 filter `!storage.zhucheng || !storage.zhucheng.length`）—— park 直推的就是 `begin:false`，此前臂写成 `begin:true` 是死代码；全量 1538 passed",
    "WudianPaotaiFire": "第 930 轮：`TriggerMeta.event` 改 `PhaseEnd`，臂改 `Phase { begin: false, .. }` + `*player == owner` —— 对齐上游 `player:'phaseEnd'`；全量 1538 passed",
    "PhaseEndRecoverOrDraw": "第 930 轮：同上（上游 `player:'phaseEnd'`）；全量 1538 passed",
    "WudianZiyu": "第 929 轮：`TriggerMeta.event` 由 `PhaseBegin` 改 `PhaseEnd`，过滤臂改 `Phase { begin: false, .. }` + `phase_number % 4 == 0`，对齐上游 `{global:'phaseEnd'}`；`nc08_upstream_ziyu` 4 项 + `g1_wudian` 122 项全绿",
}

#: 证据等级较低的一类：**「需要新事件槽位」这个前提已被推翻**，且原生与上游的效果
#: 逐分支读过一致，但**仓库里没有断言该行为的测试**。它们不计入缺陷，也不冒充「测试背书」。
NON_DEFECT_BY_CODE: dict[str, str] = {
    "XianjianChangnian": "触发用 `EventData::Death`（kernel/dying.rs 在最终死亡前发出）＝上游 `dieBegin`；效果「选一名其他角色移交 `hej` 全部牌」与上游 `chooseTarget` + `getCards('hej')` 一致（第 861 轮读码）",
    "XianjianCtianfu3": "同上窗口；效果「挂附闪入弃堆 + 清挂附 + 移除 ctianfu2/ctianfu3」与上游 `storage.ctianfu2.discard()` + `removeSkill('ctianfu2')` 一致（第 895 轮读码）",
    "XianjianXshuangren2": "同上窗口；效果「死亡时弃置 `off_court` 全部牌」与上游 `storage.xshuangren.discard()` 一致，差异「一张 → 离场区全部」已登记（第 885 轮读码）",
}

NON_DEFECT_VERIFIED: dict[str, str] = {
    "SwdDunxing": "g1_shahai_swd/heihuo.rs 断言「非红桃判定 ⇒ 杀被取消（无伤）」⇒ 取消发生在结算前",
    "SwdNingshuang": "g1_shahai_swd/yulin.rs 断言「使用者横置/未翻面 + 自己摸 1 + 伤害照常」⇒ 与上游一致",
    "SwdHuanyin": "g1_shahai_swd/huanyin.rs 断言「黑桃判定 ⇒ 铁索被取消（不横置）」与「红桃 ⇒ 摸 1 且不取消」⇒ 与上游 useCardToBefore 的取消/加牌两面一致",
    "SwdLianda": "g1_shahai_swd/lianda.rs 断言「原杀 1 伤 + 虚拟杀再结算 1 伤」「非杀无窗」⇒ 与上游 shaAfter 的再出一杀一致",
    "Yihua": "swd_yihua_bounce.rs 断言「弃 2 张手牌」「1 号未受伤（已反弹）」「0 号受伤」⇒ 反弹发生在结算前，同上游 useCardToBefore",
    "Zhenlie": "swd_zhenlie_target_exclude.rs 断言「失去 1 点体力」「使用者被弃一张牌」「未被杀命中（已移出目标）」⇒ 移出目标生效于结算前",
    "YunchouDiychanyuan": "nc08_upstream_yunchou_chanyuan.rs 断言「击杀者应失去 1 点体力上限」「体力被钳到新上限」⇒ 正是上游 `trigger.source.loseMaxHp(true)`，且发生在死亡窗口内；`EventData::Death` 在 `kernel/dying.rs` 里于最终死亡**之前**发出，等价 `dieBegin`",
    "SwdJiguanyuanGain": "上游 `card/swd/skill.js:498` 的 `jiguanyuan` 就是 `trigger:{player:'phaseEnd'}` + `forced` + `player.gain(player.storage.jiguanyuan)` ⇒ 原生 `TriggerMeta.event = PhaseEnd` + 过滤臂 `Phase{begin:false}` 一致；台账原写「damageEnd + damageSource」是**归因错误**（第 933 轮核对原文）",
    "BossBaiyi": "上游 `boss_baiyi` 是 `group:['_draw','_thunder','_discard']` 的复合技，`_discard` 子技为 `global:'roundStart'` + `roundNumber==5`；原生过滤臂判 `round == 5` + Boss 方 + `wing_armed & WING_BAI_5 == 0`（只触发一次）⇒ 同一轮内、同样一次；native_kernel/d4.rs 的 `d4_boss_b4_baiyi_discard_thunder_and_draw` 断言「第 5 轮敌方弃 2」且 wing 位已置",
    "XianjianFenglue": "过滤臂是 `Phase { Play, begin: true }` + `*player == owner`，正是上游 `player:'phaseUseBefore'`（自己出牌阶段开始前）的边界时刻；g1_wudian/luozhaoyan.rs 断言「杀/决斗自动使用、非杀不自动、杀+决斗各 1 伤」",
    "Tianxiang": "`damageBegin1/2/3/4` 是**同一次伤害内的四个阶段**（`game.js:20760-20766` 依次 `event.trigger`），不是「第 N 次伤害」⇒ 上游 `damageBegin3` 即「任意伤害、第 3 阶段」，原生单一 `DamageBefore` 窗口等价；`swd_tianxiang_redirect.rs` 断言弃红桃手牌转移伤害且天香·受按失去体力摸牌",
    "Jifeng": "swd_jifeng_multishot.rs 断言「多目标命中样本出现」与「多目标失误（整牌取消）样本出现」⇒ 取消在伤害前生效；ljifeng.rs 另断言范围扩展（对座/双目标均受 1）",
    "SwdTiebi": "filtering/swd.rs 的臂里**已含** `hujia == 0` 与 `seat_distance(t, owner) <= 1`（第 913 轮只读了前 5 行就误判为缺失）；g1_shahai_swd/tiebi.rs 断言「护甲抵 1 伤（hp 不减）」「护甲已消耗」⇒ 触发与效果都成立",
}

BARE_USE_CARD_REVIEW = {
    "SwdGongshen", "SwdLanzhi", "SwdLiuhong", "XianjianLiefeng", "NewZhixi",
}


def event_out():
    keys = phase.host_keys()
    skills = phase.skill_names()
    for family, classifier in (
        ("damage", lambda w, t: event.classify(w, t)),
        ("card", lambda w, t: event.classify_card(w, t)),
        ("death", lambda w, t: event.classify_death(w, t)),
    ):
        meta = event.metadata_events()
        if family == "damage":
            candidates = event.arms()
        elif family == "card":
            candidates = [(b, e, "meta", 0) for b, e in sorted(meta.items()) if e in {"CardUsed", "CardResponded"}]
        else:
            candidates = [(b, e, "meta", 0) for b, e in sorted(meta.items()) if e in {"Death", "Dying"}]
        for behavior, which, loc, line in candidates:
            if behavior in BARE_USE_CARD_REVIEW:
                continue
            key = keys.get(behavior, "")
            trigger = phase.resolve_trigger(behavior, key, skills.get(key, "") or "")
            # 时序权威是 TriggerMeta（过滤臂的匹配模式会产生假阳性，见枚举证据 §1）
            native = meta.get(behavior, which)
            yield behavior, native, f"{loc}:{line}", skills.get(key, "") or "", trigger, classifier(native, trigger)


def package_upstreams() -> list[tuple[str, str]]:
    """[(upstream_path 片段, 来源 id)] —— 来自包数据的 `upstream_path` 字段。

    这是**证据型归因**：缺陷的触发器在哪个上游文件里找到，就归到
    「`upstream_path` 覆盖该文件的包」所属的来源。
    """
    try:
        mapper = _load("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
        by_src, _names = mapper.mapping()
    except Exception:
        return []
    out: list[tuple[str, str]] = []
    for source, packages in by_src.items():
        for package in packages:
            for asset in (ROOT / "packages" / "extensions" / package / "assets").glob("*.json"):
                try:
                    doc = json.loads(asset.read_text(encoding="utf-8"))
                except Exception:
                    continue
                path = (doc.get("upstream_path") or "").strip()
                if path:
                    out.append((path.strip("/"), source))
    # 长的片段优先匹配（`extension/杀海拾遗/character/ow/` 先于 `extension/杀海拾遗/`）
    out.sort(key=lambda item: -len(item[0]))
    return out


def source_of_path(upstream_file: str) -> str:
    if not upstream_file:
        return "?"
    normalized = upstream_file.replace("\\", "/")
    for fragment, source in package_upstreams():
        if fragment and fragment in normalized:
            return source
    return "?"


def key_of(behavior: str) -> str:
    return phase.host_keys().get(behavior, "").removeprefix("host:")


def source_of(behavior: str) -> str:
    """behavior -> 来源 id（经 host 键 -> 包 -> 来源），查不到返回 `?`。"""
    try:
        mapper = _load("map_package_sources", ROOT / "scripts" / "map_package_sources.py")
        by_src, _name = mapper.mapping()
    except Exception:
        return "?"
    keys = phase.host_keys()
    key = keys.get(behavior, "").removeprefix("host:")
    # 1) 先按上游文件路径归因（证据型）
    _trigger, path = phase.upstream_trigger_with_path(
        phase.skill_names().get(f"host:{key}", "") or "", key
    )
    by_path = source_of_path(path)
    if by_path != "?":
        return by_path
    if not key:
        return "?"
    for source, packages in by_src.items():
        for package in packages:
            path = ROOT / "packages" / "extensions" / package / "assets"
            if not path.is_dir():
                continue
            for asset in path.glob("*.json"):
                try:
                    doc = json.loads(asset.read_text(encoding="utf-8"))
                except Exception:
                    continue
                for skill in doc.get("skills") or []:
                    if isinstance(skill, dict) and skill.get("mapping") == f"host:{key}":
                        return source
    return "?"


def by_source(rows: list[list[str]]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for row in rows:
        out.setdefault(row[3], []).append(row[0])
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", type=Path, help="write TSV to this path")
    ap.add_argument("--by-source", action="store_true", help="also print per-source counts")
    args = ap.parse_args()

    rows = build()
    header = ["id", "class", "behavior", "source", "location", "upstream", "native",
              "severity", "fix", "permission"]
    lines = ["\t".join(header)] + ["\t".join(r) for r in rows]
    text = "\n".join(lines) + "\n"
    if args.write:
        args.write.parent.mkdir(parents=True, exist_ok=True)
        args.write.write_text(text, encoding="utf-8")
        print(f"wrote {args.write} ({len(rows)} rows)", file=sys.stderr)
    else:
        print(text, end="")

    by_class: dict[str, int] = {}
    for r in rows:
        by_class[r[1]] = by_class.get(r[1], 0) + 1
    print(f"合计 {len(rows)} 条：{by_class}", file=sys.stderr)

    if args.by_source:
        counts = by_source(rows)
        print("\n来源\t条数\t缺陷族", file=sys.stderr)
        for source, ids in sorted(counts.items(), key=lambda kv: (-len(kv[1]), kv[0])):
            print(f"{source}\t{len(ids)}\t{','.join(sorted(set(ids)))}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
