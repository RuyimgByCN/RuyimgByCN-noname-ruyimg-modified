#!/usr/bin/env python3
"""Decision 34 item-by-item upstream inventory extractor for all 37 sources.

Strictly enforces Decision 35 clean-room Spine guardrail: NEVER reads or opens
`spine.js`, `spine-lib/*`, or `spine-pixi-v8.js`.
"""
from __future__ import annotations

import json
import re
from pathlib import Path

WORKSPACE = Path(__file__).resolve().parents[3]
REPO = Path(__file__).resolve().parents[2]

FORBIDDEN_FILES = {"spine.js", "spine-pixi-v8.js"}
FORBIDDEN_DIRS = {"spine-lib", "node_modules"}
VENDOR_FILES = {
    "eruda.js",
    "eruda_old.js",
    "vue.js",
    "pixi6.min.js",
    "pixi6.min_chg.js",
    "pixi8.js",
    "pixi8.min.js",
    "pixi.min.js",
    "live2dcubismcore.min.js",
    "gsap.min.js",
    "any-touch.umd.min.js",
    "better-scroll.js",
    "highlight.min.js",
    "javascript.min.js",
    "SimplePagination_old.js",
    "skinShare_old.js",
    "extension_old.js",
    "extension_mess.js",
    "main1_old.js",
}


def is_allowed_js(path: Path) -> bool:
    if path.name in FORBIDDEN_FILES or any(p in FORBIDDEN_DIRS for p in path.parts):
        return False
    if path.name in VENDOR_FILES:
        return False
    return True


def read_lines_safe(path: Path) -> list[str]:
    assert path.name not in FORBIDDEN_FILES and not any(
        p in FORBIDDEN_DIRS for p in path.parts
    ), f"Decision 35 violation: forbidden file {path}"
    return path.read_text(encoding="utf-8", errors="replace").splitlines()


def clean_md(text: str, max_len: int = 90) -> str:
    s = re.sub(r"<[^>]+>", "", text)
    s = s.replace("|", "｜").replace("\n", " ").replace("\r", " ")
    s = re.sub(r"\s+", " ", s).strip()
    if len(s) > max_len:
        return s[: max_len - 1] + "…"
    return s or "—"


def load_formal_packages() -> dict[str, dict]:
    """Load formal package definitions from packages/extensions/*."""
    pkgs: dict[str, dict] = {}
    ext_root = REPO / "packages" / "extensions"
    for d in sorted(ext_root.iterdir()):
        if not d.is_dir():
            continue
        info: dict[str, dict] = {
            "generals": {},
            "skills": {},
            "cards": {},
            "deferred": {},
            "configs": {},
        }
        assets = d / "assets"
        if assets.exists():
            for jf in sorted(assets.glob("*.json")):
                data = json.loads(jf.read_text(encoding="utf-8"))
                if jf.name in ("cards.json", "character.json", "characters.json"):
                    for c in data.get("cards", []):
                        info["cards"][c["name"]] = c.get("kind", "basic")
                    for dd in data.get("deferred", []):
                        info["deferred"][dd["name"]] = dd.get("reason", "deferred")
                    for sk in data.get("skills", []):
                        info["skills"][sk["name"]] = sk.get("mapping")
                    for g in data.get("generals", []):
                        gid = g.get("id") or g.get("name")
                        g_skills = {}
                        for sk in g.get("skills", []):
                            g_skills[sk["name"]] = sk.get("mapping")
                            if sk["name"] not in info["skills"] or sk.get("mapping"):
                                info["skills"][sk["name"]] = sk.get("mapping")
                        info["generals"][gid] = g_skills
                elif jf.name in ("config.json", "pile.json", "wuxing.json"):
                    if isinstance(data, dict):
                        for k, v in data.items():
                            info["configs"][k] = str(v)[:40]
        pkgs[d.name] = info
    return pkgs


SOURCE_FORMAL_PKGS: dict[str, list[str]] = {
    "U01": ["d3-upstream-noname"],
    "U02": ["boss-play-upstream-noname"],
    "U03": ["cardpile-upstream-noname"],
    "U04": ["coin-upstream-noname"],
    "U05": [
        "shahai-gujian-upstream-noname",
        "shahai-gwent-upstream-noname",
        "shahai-hearth-upstream-noname",
        "shahai-mtg-upstream-noname",
        "shahai-swd-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
        "shaai-ow-upstream-noname",
        "shaai-xianjian-upstream-noname",
    ],
    "U06": ["huanle-upstream-noname"],
    "U07": ["wudian-upstream-noname"],
    "U08": ["hero-kill-upstream-noname"],
    "L01": ["ai-tuning-upstream-liuli"],
    "L02": ["sj-settings-upstream-liuli"],
    "L03": ["boss-play-upstream-liuli", "boss-zhugeliang-upstream-liuli"],
    "L04": ["cardpile-upstream-liuli"],
    "L05": ["coin-upstream-liuli"],
    "L06": ["wuxing-upstream-liuli"],
    "L07": ["omnisearch-upstream-liuli"],
    "L08": ["decade-ui-upstream-liuli"],
    "L09": ["qhly-upstream-liuli"],
    "L10": ["epic-cards-upstream-liuli"],
    "L11": ["taixu-upstream-liuli"],
    "L12": ["rzsh-upstream-liuli"],
    "L13": ["shousha-mvp-upstream-liuli"],
    "L14": ["shousha-ui-upstream-liuli"],
    "L15": ["shousha-full-upstream-liuli"],
    "L16": ["drag-read-upstream-liuli"],
    "L17": ["douzhuan-xingyi-upstream-liuli"],
    "L18": ["update-guard-upstream-liuli"],
    "L19": ["doomsday-brawl-upstream-liuli"],
    "L20": ["mark-supplement-upstream-liuli"],
    "L21": ["martial-upstream-liuli"],
    "L22": ["elo-sanction-upstream-liuli"],
    "L23": ["desktop-adapt-upstream-liuli"],
    "L24": ["skin-switch-upstream-liuli"],
    "L25": ["zuan-upstream-liuli"],
    "L26": ["fool-plugin-upstream-liuli"],
    "L27": ["select-beautify-upstream-liuli"],
    "L28": ["thunder-upstream-liuli"],
    "L29": ["longxue-upstream-liuli"],
}


DECISION34_AUDIT: dict[str, list[tuple[str, str, str, str, str]]] = {
    "U01": [
        (
            "`U01_DDD_GENERALS`（31 名 `ddd_*` 武将，已接入正式包 `d3-upstream-noname` 产品路径）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:10-42`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:5-215`",
            "原生规则 / `d3-upstream-noname`",
            "部分忠实（31 名武将已通过 `attach_formal_package_upstream_rules` 接入 `d3-upstream-noname` 正式包产品路径；其中 `ddd_zhangmiao`、`ddd_caomao` 等所含 8 项技能已忠实实现，其余 56 项技能待逐个实现）",
        ),
        (
            "`U01_DDD_SKILLS` + `resolve_u01_ddd_skill`（64 项 `ddd*` 技能）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:141-294`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`",
            "原生规则 / `d3-upstream-noname`",
            "部分忠实（已清除 `_ => ctx.start_draw(owner, 1)` 兜底并按上游忠实实现 `dddxiaxing`、`dddyeshen`、`dddqiaoduan`、`dddqianlong`、`dddqice`、`dddjungui`、`dddshilie`、`ddddiedang` 8 项技能且接入正式包；其余 56 项技能仅记录已触发标志）",
        ),
        (
            "`[data-testid=\"u01-3d-select-gallery\"]` 3D 镜头展台",
            "`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:672-691`",
            "上游无独立 3D 镜头展台（仅 `character/character.js:218` 设置立绘路径）",
            "原生前端组件",
            "示意或自造（前端自造展台控件）",
        ),
    ],
    "U02": [
        (
            "`boss_stage_transition` / `resolve_boss_stage_transition`（`boss_baonu_stage` 已按第二步清除）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:403-416`",
            "`archive/upstream/noname/apps/core/extension/boss/extension.js:6-146`（上游仅做挑战/剑阁武将池缓存，无此技能）",
            "原生规则",
            "示意或自造（已清除伪造的 `boss_baonu_stage`；`boss_stage_transition` 仅在测试包注册）",
        ),
        (
            "`stepBossStage` 前端闯关按钮",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:436-463`",
            "上游 `U02` 为 `play` 角色池加载器，无独立闯关面板",
            "原生前端组件",
            "示意或自造（前端自造战役按钮）",
        ),
    ],
    "U03": [],
    "U04": [
        (
            "`resolveCoinHonorTitle` / `[data-testid=\"coin-weather-overlay\"]`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:392-434`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:438-480`",
            "`archive/upstream/noname/apps/core/extension/coin/extension.js:17-540`",
            "原生前端组件",
            "示意或自造（CSS 示意粒子层，未复刻上游 `canvas` 烟花/下雪/星云/闪烁与窗口拖拽买道具交互）",
        ),
    ],
    "U05": [
        (
            "`U05_SWD_REMAINING_SKILLS` / `U05_YUNCHOU_REMAINING_SKILLS` / `U05_MTG_LAND_CARDS` / `U05_ZHENFA_CARDS` / `U05_OW_SKILLS` / `U05_XIANJIAN_GENERALS` / `U05_XIANJIAN_SKILLS`",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:60-124`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:299-534`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/` 各子目录",
            "原生规则 / `packages/extensions/sha*`",
            "部分忠实（已清除 `resolve_ow_skill` 与 `resolve_xianjian_skill` 的 `_ => ctx.start_draw(owner, 1)` 兜底并通过 `attach_formal_package_upstream_rules` 接入 `packages/extensions/sha*` 正式包；`character/{gujian,gwent,hearth,mtg}` 武将子目录待迁入）",
        ),
    ],
    "U06": [
        (
            "`huanle_kuwu_interactive` / `resolve_huanle_kuwu_interactive`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:42-137`",
            "`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:53-69`",
            "原生规则 / `huanle-upstream-noname`",
            "忠实实现（已接入正式包 `huanle-upstream-noname` 产品路径，支持弃牌分支与失去 1 点体力拿走苦无分支）",
        ),
    ],
    "U07": [
        (
            "`diaobingqianjiang` / `duoqi` / `youzhan` (`decision34.rs`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:144-272`",
            "`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:4-168`、`character/skill.js:109-159,422-447`",
            "原生规则 / `wudian-upstream-noname`",
            "示意或自造（已接入正式包 `wudian-upstream-noname` 产品路径，但简化了交互与触发条件）",
        ),
        (
            "`jianbi` / `choudu` (`decision34_pack9.rs`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:539-550`",
            "`archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js:586-615,726-775`",
            "原生规则 / `wudian-upstream-noname`",
            "示意或自造（已接入正式包 `wudian-upstream-noname`，但 `jianbi`/`choudu` 仍为简化分支）",
        ),
    ],
    "U08": [
        (
            "`yxs_menshen2_guard` / `verify_u08_standard_skill_bindings`（伪造 `yxs_tafang_wave` 与 `advanceTafangWave` 已按第二步清除）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:278-309`、`crates/content-native-loader/src/decision34_closure.rs:348-386`",
            "`archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:87-109`、`character/character.js:3-287`",
            "原生规则 / `hero-kill-upstream-noname`",
            "部分忠实（已清除伪造的 `yxs_tafang_wave` 与 `advanceTafangWave`，`yxs_menshen2_guard` 已接入正式包 `hero-kill-upstream-noname`；35 名 `yxs_*` 武将与 108 个自带技能待迁入）",
        ),
    ],
    "L01": [],
    "L02": [],
    "L03": [
        (
            "`jiange_siege_step` / `resolve_jiange_siege_step`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:421-432`",
            "`archive/upstream/noname-liuli/noname/extension/boss/extension.js:5-136`（上游仅做挑战/剑阁武将池缓存，无此技能）",
            "原生规则",
            "示意或自造（仅在测试包注册的合成技能）",
        ),
    ],
    "L04": [],
    "L05": [],
    "L06": [],
    "L07": [
        (
            "`runOmnisearchQuery` / `[data-testid=\"omnisearch-panel\"]`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:523-568`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:526-567`",
            "`archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js:1-2354`",
            "原生前端组件",
            "示意或自造（仅含固定几条硬编码词条与简单卡片，未连接完整 `PublicIndex` 分页/高亮/详情弹窗）",
        ),
    ],
    "L08": [
        (
            "`L08_DECADE_CHARACTERS`（9 名伪造 `decade_*` 武将已按第二步清除）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:127-130`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12687-12692`（上游 `character: {}` 为空）",
            "无（已清除）",
            "已清除（9 名伪造 `decade_*` 武将与 `_decade_qiankunbagua` 已从代码删除，见 `docs/operations/evidence/2026-11-09-decision34-cleanup/README.md`）",
        ),
        (
            "`L08_DECADE_SKILLS` + `resolve_l08_decade_skill`（9 项 `_changeEquipExt`/`_usecard` 等 UI 钩子）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:555-566`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:54-650`",
            "原生前端组件",
            "示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS/Spine 特效待在前端组件实现）",
        ),
        (
            "净室 Spine 4.0 播放器 (`apps/web-client/src/engine/spine40/*`，已替换示意 `drawSpineSkeletalCanvas`)",
            "`apps/web-client/src/engine/spine40/webglRenderer.ts:1-406`、`apps/web-client/src/engine/spine40/index.ts:1-5`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:353-388`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816`、`dynamicSkin.js:1-162`",
            "原生前端组件（净室 Spine 4.0 播放器）",
            "忠实实现（按裁决 35 净室自写 Spine 4.0 JSON/二进制解码、骨骼世界变换、IK/变换/路径约束、多轨道混合、图集解析与 WebGL PMA 渲染，无动皮资源或非 4.0 资源时回退静态立绘）",
        ),
    ],
    "L09": [
        (
            "`switchQhlySkin` / `switchQhlyTheme`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:654-685`",
            "`archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:1-15970`",
            "原生前端组件",
            "示意或自造（仅状态字段切换，未完整复刻千幻皮肤详情/台词/语音试听窗口与 `saveSkinParams` 参数导入）",
        ),
    ],
    "L10": [
        (
            "`k_danggu` / `k_mowang` (`resolve_l10_k_danggu`, `resolve_l10_k_mowang`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:318-397`",
            "`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2276-2970`",
            "原生规则 / `epic-cards-upstream-liuli` + 原生前端组件",
            "示意或自造（已接入正式包 `epic-cards-upstream-liuli` 产品路径，但仍为简化版，未按上游结党/十侍子技能完整授予）",
        ),
    ],
    "L11": [
        (
            "`taixu_expedition_step`（伪造步进技能已按第二步清除）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:435-445`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension.js:1-6461` 及 `dlc/*`、`extension_*.js`",
            "原生规则 / `taixu-upstream-liuli` + 原生前端组件",
            "已清除伪造步进技能（`taixu_expedition_step` 已删除；上游 8 赛季 DLC、319 敌将、61 牌、8 侍灵与关卡事件待实现）",
        ),
    ],
    "L12": [
        (
            "`toggleRzshTheme` / `numToRoma` 面板",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:687-708`",
            "`archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:1-9502`、`js/rzsh.js:1-815`",
            "原生前端组件",
            "示意或自造（`numToRoma` 函数忠实，但大厅/排位/动皮界面仅为抽屉内按钮）",
        ),
    ],
    "L13": [],
    "L14": [
        (
            "`L14_SHOUSHA_SKILLS` + `resolve_l14_shousha_skill`（伪造武将/卡牌已清除，48 个 UI 钩子已清零摸牌/回血副作用）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:132-143`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:571-583`",
            "`archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-7360`",
            "原生前端组件",
            "示意或自造（已删除伪造 `shousha_*` 武将/卡牌及摸牌/回血副作用；手杀进度条、菜单、托管、语音气泡等前端视觉表现待完整复刻）",
        ),
    ],
    "L15": [],
    "L16": [],
    "L17": [],
    "L18": [],
    "L19": [
        (
            "`L19_DOOMSDAY_CHARACTERS`（已清除 22 名伪造 `vir_*` 武将，保留上游 `jiangshifz` 与 `jiangshinj` 并接入正式包）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:145-147`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-332,3383-3385`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "忠实实现（已清除 22 名伪造 `vir_*` 武将与 `doomsday_infection_step`；`jiangshifz` 与 `jiangshinj` 已接入正式包 `doomsday-brawl-upstream-liuli` 产品路径）",
        ),
        (
            "`L19_DOOMSDAY_SKILLS` + `resolve_l19_doomsday_skill`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:588-617`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:333-3370`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "部分忠实（已清除 `_ => ctx.start_draw(owner, 1)` 兜底并接入正式包；`baozou`、`ganran`、`shishi`、`xunmeng` 已实现，其余部分模式钩子待补完）",
        ),
    ],
    "L20": [
        (
            "`L20_MARK_SUPPLEMENT_CARDS`（已清除 12 名伪造 `ms_*` 武将与 `ms_longmark`，保留 3 张 `zhengsu_*` 牌并接入正式包）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:166-168`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321,6493-6505`",
            "原生规则 / `mark-supplement-upstream-liuli`",
            "忠实实现（已清除 12 名伪造 `ms_*` 武将与 `ms_longmark`；上游 3 张 `zhengsu_*` 衍生牌已接入正式包 `mark-supplement-upstream-liuli` 产品路径）",
        ),
        (
            "`L20_MARK_SUPPLEMENT_SKILLS` + `resolve_l20_skill` + `resolve_l20_hud_mark_sync`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:101-133,622-667`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:577-6480`",
            "原生规则 / `mark-supplement-upstream-liuli` + 原生前端组件",
            "示意或自造（已清除 `_ => ctx.start_draw(owner, 1)` 兜底并接入正式包 `mark-supplement-upstream-liuli`；其余标记特效钩子待在前端完整复刻）",
        ),
    ],
    "L21": [
        (
            "`martial_bench_substitute` / `resolve_martial_bench_substitute`（伪造 `martial_formation_step` 已按第二步清除）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:437-458`",
            "`archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js:1-1367`",
            "原生规则 / `martial-upstream-liuli`",
            "示意或自造（已清除伪造 `martial_formation_step`；真实武皇争霸逻辑已在 `content-native-loader::martial` 实现）",
        ),
    ],
    "L22": [],
    "L23": [],
    "L24": [
        (
            "`L24_SKIN_SWITCH_SKILLS` + `resolve_l24_skin_skill`（23 项皮肤/语音/出框 UI 钩子，已清零摸牌副作用）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:188-194`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:672-682`",
            "`archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-5900`",
            "原生前端组件",
            "示意或自造（已清除规则层摸牌副作用，仅递增 `_l24_skin_fx_seq`；Spine 4.0 净室播放器已落地，完整出框/语音/参数编辑界面待补完）",
        ),
    ],
    "L25": [
        (
            "`L25_ZUAN_CHARACTERS`（已清除 60 名伪造 `zuan_*` 武将，保留上游 60 名 `ssr_*` 武将并接入正式包）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:196-211`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-397`",
            "原生规则 / `zuan-upstream-liuli`",
            "部分忠实（已清除 60 名伪造 `zuan_*` 武将与 `zuan_jianxiong`；60 名 `ssr_*` 武将已接入正式包 `zuan-upstream-liuli` 产品路径，自带 `ssr_*` 技能待逐个实现）",
        ),
        (
            "`L25_ZUAN_SKILLS` + `resolve_l25_zuan_skill`",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:213-239`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:687-708`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:536-51470`",
            "原生规则 / `zuan-upstream-liuli`",
            "示意或自造（已清除 `_ => ctx.start_draw(owner, 1)` 兜底与 DIY 钩子摸牌；`package.skill.skill` 内 230 项 `ssr_*` 技能待实现）",
        ),
    ],
    "L26": [],
    "L27": [
        (
            "`L27_SELECT_BEAUTIFY_SKILLS` + `resolve_l27_select_skill`",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:241-250`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:713-736`",
            "`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:1162-9090`",
            "原生规则 / `select-beautify-upstream-liuli` + 原生前端组件",
            "示意或自造（已清除 `_chooseCharacterTwo` 摸 2 张牌与 `_ => ctx.start_draw(owner, 1)` 兜底并接入正式包 `select-beautify-upstream-liuli`；其余技能钩子待补完）",
        ),
    ],
    "L28": [
        (
            "`L28_THUNDER_GENERALS`（已清除 11 名伪造武将与重奏/幻音/旋律，按上游实现 14 名 `th_*` 武将、技能与 `th_taipingyaoshu` 并接入正式包）",
            "`crates/content-native-loader/src/decision34_closure.rs:220-264`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:463-699`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3656-3671,3781-8968`",
            "原生规则 / `thunder-upstream-liuli`",
            "部分忠实（已清除伪造 `th_caocao` 等与 `th_chongzou`/`th_huanyin`/`th_xuanlv`，按上游忠实实现 `th_dunshi`、`th_jingxie`、`th_qiaosi`、`th_yeyan`、`th_taipingyaoshu` 及 14 名 `th_*` 武将并接入正式包 `thunder-upstream-liuli` 产品路径；其余 17 项技能待完整实现）",
        ),
    ],
    "L29": [
        (
            "`mbqianlong_refresh` / `mbcmqingzheng` / `mbcmjiushi` / `mbcmfangzhu` / `mbjuejin` (`decision34_pack9.rs`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:14-96`、`crates/content-native-loader/src/locked_content.rs:114-119`",
            "`docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1`、`archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.js:1-1048`",
            "原生规则 / `longxue-upstream-liuli`",
            "部分忠实（`mbqianlong` 已在正式包映射 `host:mbqianlong`，`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin` 已通过 `attach_formal_package_upstream_rules` 接入正式包 `longxue-upstream-liuli` 产品路径）",
        ),
    ],
}

RESERVED_SKILL_KEYS = {
    "trigger", "filter", "content", "check", "ai", "mod", "init", "onremove",
    "subSkill", "group", "marktext", "intro", "derivation", "viewAs", "filterCard",
    "selectCard", "filterTarget", "selectTarget", "position", "prompt", "usable",
    "round", "forced", "locked", "direct", "popup", "audio", "skillAnimation",
    "animationColor", "unique", "charlotte", "persevereSkill", "zhuSkill",
    "juexingji", "limited", "dutySkill", "hiddenSkill", "nopop", "frequent",
    "nobracket", "firstDo", "lastDo", "inherit", "effect", "result", "basic",
    "value", "useful", "order", "threaten", "expose", "tag", "skillTagFilter",
    "global", "chooseButton", "backup", " custom", "cost", "createDialog",
    "player", "target", "source", "cards", "card", "skill", "character", "translate",
    "list", "pile", "config", "help", "package", "files", "dynamicTranslate",
    "Precontent", "precontent", "arenaReady", "prepare", "start", "startBefore",
    "element", "game", "ui", "get", "lib", "status", "_status",
}

DECISION34_SKILL_SET = {
    "mbcmjiushi", "mbcmfangzhu", "mbjuejin", "k_danggu", "k_mowang",
    "huanle_kuwu_interactive", "diaobingqianjiang", "duoqi", "youzhan",
    "yxs_menshen2_guard", "yxs_tafang_wave", "l20_mark_dispatch",
    "th_chongzou", "th_huanyin", "th_xuanlv", "boss_baonu_stage",
    "taixu_expedition_step", "doomsday_infection_step", "martial_formation_step",
    "jianbi", "choudu", "duijue", "tanlin", "ycndiyjianxiong", "xinfu_jingxie",
    "ycnxinfu_qiaosi",
}

UI_ONLY_SOURCES = {"L07", "L08", "L09", "L12", "L13", "L14", "L16", "L18", "L23", "L24", "L26"}
PORTABLE_POLICY_SOURCES = {
    "U02": "crates/content-native-loader/src/boss_playpack.rs",
    "U03": "crates/content-native-loader/src/pile_bridge.rs",
    "U04": "crates/content-native-loader/src/coin.rs",
    "L01": "crates/content-native-loader/src/ai_tuning.rs",
    "L02": "crates/content-native-loader/src/sj_settings.rs",
    "L03": "crates/content-native-loader/src/boss_playpack.rs",
    "L04": "crates/content-native-loader/src/pile_bridge.rs",
    "L05": "crates/content-native-loader/src/coin.rs",
    "L06": "packages/extensions/wuxing-upstream-liuli",
    "L07": "crates/content-native-loader/src/omnisearch.rs",
    "L08": "crates/content-native-loader/src/decade_ui.rs",
    "L09": "crates/content-native-loader/src/qhly.rs",
    "L10": "crates/content-native-loader/src/epic_cards.rs",
    "L11": "crates/content-native-loader/src/taixu.rs",
    "L12": "crates/content-native-loader/src/rzsh.rs",
    "L13": "crates/content-native-loader/src/shousha_mvp.rs",
    "L14": "crates/content-native-loader/src/shousha_ui.rs",
    "L15": "crates/content-native-loader/src/shousha.rs",
    "L16": "crates/content-native-loader/src/drag_read.rs",
    "L17": "crates/content-native-loader/src/douzhuan.rs",
    "L18": "crates/content-native-loader/src/update_guard.rs",
    "L19": "crates/content-native-loader/src/doomsday.rs",
    "L21": "crates/content-native-loader/src/martial.rs",
    "L22": "crates/content-native-loader/src/elo.rs",
    "L23": "crates/content-native-loader/src/desktop_adapt.rs",
    "L24": "crates/content-native-loader/src/skin_switch.rs",
    "L25": "crates/content-native-loader/src/zuan.rs",
    "L26": "crates/content-native-loader/src/fool.rs",
    "L28": "crates/content-native-loader/src/thunder.rs",
    "L29": "crates/content-native-loader/src/longxue.rs",
}


def rel_to_workspace(p: Path) -> str:
    try:
        return p.relative_to(WORKSPACE).as_posix()
    except ValueError:
        return p.relative_to(REPO).as_posix()


def extract_translations(js_files: list[Path]) -> dict[str, str]:
    trans: dict[str, str] = {}
    re_kv = re.compile(r"""^\s*["']?([A-Za-z0-9_$\u4e00-\u9fa5]+)["']?\s*:\s*["']([^"']+)["']""")
    re_assign = re.compile(
        r"""lib\.translate(?:\.([A-Za-z0-9_$]+)|\[["']([A-Za-z0-9_$\u4e00-\u9fa5]+)["']\])\s*=\s*["']([^"']+)["']"""
    )
    for jf in js_files:
        lines = read_lines_safe(jf)
        in_trans = jf.name == "translate.js"
        for line in lines:
            m_a = re_assign.search(line)
            if m_a:
                k = m_a.group(1) or m_a.group(2)
                trans[k] = m_a.group(3)
                continue
            if "translate:" in line and "{" in line:
                in_trans = True
            if in_trans:
                m_kv = re_kv.match(line)
                if m_kv:
                    k, v = m_kv.group(1), m_kv.group(2)
                    if k not in trans:
                        trans[k] = v
    return trans


def summarize_snippet(lines: list[str], start_idx: int, trans: dict[str, str], key: str) -> str:
    zh = trans.get(key, "")
    info = trans.get(f"{key}_info", "")
    if zh and info:
        return clean_md(f"{zh}：{info}")
    window = " ".join(lines[start_idx : min(len(lines), start_idx + 18)])
    m_intro = re.search(r"""(?:intro|name|prompt)\s*:\s*["']([^"']{2,100})["']""", window)
    if m_intro:
        desc = m_intro.group(1)
        return clean_md(f"{zh}（{desc}）" if zh else desc)
    m_trig = re.search(
        r"""trigger\s*:\s*\{\s*(?:player|global|source|target)\s*:\s*([^\}]+)\}""", window
    )
    trig_s = f"触发={clean_md(m_trig.group(1), 35)}" if m_trig else ""
    effects = []
    for kw, label in [
        ("draw(", "摸牌"),
        ("recover(", "回复"),
        ("damage(", "伤害"),
        ("loseHp(", "失去体力"),
        ("chooseToDiscard(", "弃牌"),
        ("discard(", "弃置"),
        ("gain(", "获得牌"),
        ("judge(", "判定"),
        ("turnOver(", "翻面"),
        ("link(", "横置"),
        ("changeHujia(", "护甲"),
        ("addMark(", "加标记"),
        ("removeMark(", "移标记"),
        ("addTempSkill(", "临时技"),
        ("addSkill(", "获得技能"),
        ("useCard(", "使用牌"),
        ("chooseTarget(", "选目标"),
        ("chooseControl(", "选选项"),
        ("chooseButton(", "选按钮"),
        ("playAudio(", "音效"),
        ("setBackgroundImage(", "换背景/立绘"),
        ("decadeUI", "十周年UI渲染"),
    ]:
        if kw in window:
            effects.append(label)
    eff_s = "、".join(effects[:4]) if effects else "脚本逻辑"
    parts = [p for p in [zh, trig_s, eff_s] if p]
    return clean_md("；".join(parts))


def extract_source_inventory(sid: str, src_dir: Path, formal_pkgs: dict[str, dict]) -> dict[str, list[dict]]:
    """Extract all 7 categories of items from an upstream source directory."""
    js_files: list[Path] = []
    css_files: list[Path] = []
    if src_dir.exists():
        for p in sorted(src_dir.rglob("*")):
            if p.is_file():
                if p.suffix == ".js" and is_allowed_js(p):
                    js_files.append(p)
                elif p.suffix == ".css" and not any(d in FORBIDDEN_DIRS for d in p.parts):
                    css_files.append(p)
    if sid == "L29":
        readable = REPO / "docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js"
        if readable.exists():
            js_files.insert(0, readable)

    trans = extract_translations(js_files)

    pkg_generals: dict[str, dict] = {}
    pkg_skills: dict[str, str | None] = {}
    pkg_cards: dict[str, str] = {}
    pkg_deferred: dict[str, str] = {}
    for pname in SOURCE_FORMAL_PKGS.get(sid, []):
        pinfo = formal_pkgs.get(pname, {})
        pkg_generals.update(pinfo.get("generals", {}))
        for sk, mp in pinfo.get("skills", {}).items():
            if sk not in pkg_skills or mp:
                pkg_skills[sk] = mp
        pkg_cards.update(pinfo.get("cards", {}))
        pkg_deferred.update(pinfo.get("deferred", {}))

    generals: list[dict] = []
    skills: list[dict] = []
    cards: list[dict] = []
    modes: list[dict] = []
    configs: list[dict] = []
    ui_hooks: list[dict] = []
    surfaces: list[dict] = []

    seen_generals: set[str] = set()
    seen_skills: set[str] = set()
    seen_cards: set[str] = set()
    seen_configs: set[str] = set()

    re_char_arr = re.compile(
        r"""^\s*["']?([A-Za-z0-9_]+)["']?\s*:\s*\[\s*["'](male|female|double|none)["']\s*,\s*["']([A-Za-z0-9_]+)["']\s*,\s*([0-9/'"A-Za-z_]+)\s*,\s*(?:window\.getStrength\()?\[([^\]]*)\]"""
    )
    re_lib_char = re.compile(
        r"""lib\.character(?:\.([A-Za-z0-9_]+)|\[["']([A-Za-z0-9_]+)["']\])\s*=\s*\["""
    )
    re_lib_skill = re.compile(
        r"""lib\.skill(?:\.([A-Za-z0-9_$]+)|\[["']([A-Za-z0-9_$]+)["']\])\s*=\s*\{"""
    )
    re_lib_card = re.compile(
        r"""lib\.card(?:\.([A-Za-z0-9_$]+)|\[["']([A-Za-z0-9_$]+)["']\])\s*=\s*\{"""
    )
    re_char_intro = re.compile(
        r"""lib\.characterIntro\.([A-Za-z0-9_]+)\s*=\s*lib\.characterIntro\.([A-Za-z0-9_]+)"""
    )

    for jf in js_files:
        rel = rel_to_workspace(jf)
        lines = read_lines_safe(jf)
        fname = jf.name

        # Surface registration for pure UI/effect submodules
        if fname in (
            "component.js", "effect.js", "dynamicSkin.js", "animation.js",
            "animation_new.js", "meihua.js", "splash.js", "pause.js",
            "menu.js", "animationlist.js", "content.js", "SimplePagination.js",
            "skinShare.js", "play.js", "qhly.js", "rzsh.js",
        ) or any(part in ("lbtn", "character", "skill") for part in jf.parts if sid in ("L14", "L24", "L29")):
            if sid in ("L08", "L09", "L12", "L14", "L24", "L27", "L29"):
                surfaces.append({
                    "id": jf.stem,
                    "ref": f"`{rel}:1-{len(lines)}`",
                    "summary": f"前端脚本模块 `{fname}`（{len(lines)} 行）",
                    "target": "原生前端组件",
                    "status": "示意或自造（仅抽屉摘要面板，未按上游同场景完整渲染）",
                })

        for idx, line in enumerate(lines):
            lineno = idx + 1
            # 1) Character array syntax
            m_ca = re_char_arr.match(line)
            if m_ca:
                cid, sex, grp, hp, sk_raw = m_ca.groups()
                if cid not in seen_generals and not cid.startswith("//"):
                    seen_generals.add(cid)
                    sk_list = [s.strip().strip("'\"") for s in sk_raw.split(",") if s.strip().strip("'\"")]
                    generals.append(_classify_general(sid, cid, f"`{rel}:{lineno}`", sex, grp, hp, sk_list, trans, pkg_generals))
                continue
            m_lc = re_lib_char.search(line)
            if m_lc:
                cid = m_lc.group(1) or m_lc.group(2)
                if cid not in seen_generals:
                    seen_generals.add(cid)
                    generals.append(_classify_general(sid, cid, f"`{rel}:{lineno}`", "?", "?", "?", [], trans, pkg_generals))

            # 2) U02 / L03 boss playpack hooks & characterIntro mappings
            if sid in ("U02", "L03"):
                m_ci = re_char_intro.search(line)
                if m_ci:
                    b_id, base_id = m_ci.groups()
                    if b_id not in seen_generals:
                        seen_generals.add(b_id)
                        generals.append({
                            "id": b_id,
                            "ref": f"`{rel}:{lineno}`",
                            "summary": f"剑阁/挑战武将简介映射 `{b_id}` → `{base_id}`（`boss_storage_playpackconfig`）",
                            "target": "原生规则（`boss_playpack.rs`）",
                            "status": "忠实实现",
                        })
                if line.strip().startswith("init()"):
                    modes.append({
                        "id": "boss_play_init",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "加载 `boss_storage_playpackconfig` 挑战/剑阁武将池、归一化体力与 AI 禁选",
                        "target": "原生规则（`boss_playpack.rs`）",
                        "status": "忠实实现",
                    })
                elif line.strip().startswith("arenaReady()"):
                    modes.append({
                        "id": "boss_play_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "异步加载 `boss`/`versus` 模式武将池与技能并排除 `versus_ladder`",
                        "target": "原生规则（`boss_playpack.rs`）",
                        "status": "忠实实现",
                    })

            # 2b) U03 / L04 cardpile ratio entries
            if sid in ("U03", "L04"):
                m_suit = re.match(r"^\s*(diamond|club|heart|spade)\s*:\s*(\d+)\s*,", line)
                if m_suit:
                    suit, cnt = m_suit.groups()
                    cards.append({
                        "id": f"pile_{suit}_{lineno}",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": f"牌堆花色基数权重 `{suit}: {cnt}`（按 `cardpile_*_playpackconfig` 缩放）",
                        "target": "原生规则（`pile_bridge.rs`）",
                        "status": "忠实实现",
                    })
                elif line.strip().startswith("arenaReady()"):
                    modes.append({
                        "id": "cardpile_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "按 `data` 160 张基数与比例配置向 `lib.card.list` 追加牌堆条目（上限 1000）",
                        "target": "原生规则（`pile_bridge.rs`）",
                        "status": "忠实实现",
                    })

            # 2c) U04 / L05 coin shop & hooks
            if sid in ("U04", "L05"):
                if line.strip().startswith("init()"):
                    modes.append({
                        "id": "coin_init",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "非战棋统率模式下初始化 `_status.coin = 0`",
                        "target": "原生规则（`coin.rs`）",
                        "status": "忠实实现",
                    })
                elif line.strip().startswith("arenaReady()"):
                    ui_hooks.append({
                        "id": "coin_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "创建顶栏富甲天下金币按钮、商店弹窗与本局获胜 20 金下注入口",
                        "target": "原生规则（`coin.rs`）+ 原生前端组件",
                        "status": "忠实实现（规则结算已在 `coin.rs` 落地）",
                    })
                elif line.strip().startswith("changeCoin("):
                    modes.append({
                        "id": "game.changeCoin",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "增减金币、持久化 `lib.config.coin`、触发提示与音效",
                        "target": "原生规则（`coin.rs`）",
                        "status": "忠实实现",
                    })
                for shop_key, shop_zh in [("firework", "烟花"), ("snow", "下雪"), ("star", "星云"), ("blink", "闪烁")]:
                    if re.match(rf"^\s*{shop_key}\s*:\s*\{{", line) and f"shop_{shop_key}" not in seen_configs:
                        seen_configs.add(f"shop_{shop_key}")
                        surfaces.append({
                            "id": f"haveFun.{shop_key}",
                            "ref": f"`{rel}:{lineno}`",
                            "summary": f"富甲天下商店特效 `{shop_key}`（{shop_zh} Canvas 粒子与启停控制）",
                            "target": "原生规则（`coin.rs`）+ 原生前端组件",
                            "status": "示意或自造（商店购买状态机已忠实实现，前端 Canvas 粒子为 CSS 示意层）",
                        })

            # 2d) L15 手杀全扩 / L17 斗转星移 policy hooks
            if sid == "L15" and "var changePlayer=" in line:
                modes.append({
                    "id": "changePlayer_pool",
                    "ref": f"`{rel}:{lineno}`",
                    "summary": "手杀全扩白名单将池表（风林火山/山阴雷/谋攻篇等）与将包映射过滤",
                    "target": "原生规则（`shousha.rs`）",
                    "status": "忠实实现",
                })
            if sid == "L17" and "function banCharF()" in line:
                ui_hooks.append({
                    "id": "banCharF",
                    "ref": f"`{rel}:{lineno}`",
                    "summary": "斗转星移禁将面板（3 套方案切换、全禁/全开、导入导出、星级评分）",
                    "target": "原生规则（`douzhuan.rs`）+ 原生前端组件",
                    "status": "忠实实现（方案算法与导出已在 `douzhuan.rs` 落地）",
                })

            # 3) Explicit lib.skill.X = {
            m_ls = re_lib_skill.search(line)
            if m_ls:
                sk = m_ls.group(1) or m_ls.group(2)
                if sk not in seen_skills and sk not in RESERVED_SKILL_KEYS:
                    seen_skills.add(sk)
                    item = _classify_skill_or_hook(sid, sk, f"`{rel}:{lineno}`", lines, idx, trans, pkg_skills)
                    if item["is_hook"]:
                        ui_hooks.append(item)
                    else:
                        skills.append(item)

            # 4) Explicit lib.card.X = {
            m_lcard = re_lib_card.search(line)
            if m_lcard:
                cd = m_lcard.group(1) or m_lcard.group(2)
                if cd not in seen_cards and cd not in RESERVED_SKILL_KEYS:
                    seen_cards.add(cd)
                    cards.append(_classify_card(sid, cd, f"`{rel}:{lineno}`", lines, idx, trans, pkg_cards, pkg_deferred))

            # 5) Mode / campaign nodes
            if "game.addMode(" in line:
                m_mode = re.search(r"""game\.addMode\(\s*["']([^"']+)["']""", line)
                mname = m_mode.group(1) if m_mode else "mode"
                modes.append({
                    "id": mname,
                    "ref": f"`{rel}:{lineno}`",
                    "summary": f"注册游戏模式 `{mname}`（{trans.get(mname, mname)}）",
                    "target": "原生规则",
                    "status": "忠实实现" if sid in ("L21",) else "缺失（独立模式主循环未完整迁入正式包）",
                })

            # 6) L29 龙血玄黄 single-line readable JS extraction
            if sid == "L29" and fname == "extension.readable.js":
                for l29_sk, l29_desc, l29_st in [
                    ("mbqianlong", "潜龙（道心值系统主技能，含 1~6 子技能：开局选志向、获得/失去牌与受伤伤害增减道心、按 25/50/75/99 阈值动态解锁清正/酒诗/放逐/绝进）", "忠实实现（正式包已映射 `host:mbqianlong`）"),
                    ("mbcmqingzheng", "清正（道心 >= 25 解锁：出牌阶段开始弃置一种花色手牌并观看目标手牌弃置一花色，若少则造成伤害）", "示意或自造（正式包已映射 `host:mbcmqingzheng`，`decision34_pack9` 另有简化分支）"),
                    ("mbcmjiushi", "酒诗（道心 >= 50 解锁：翻转武将牌视为使用酒，受伤翻回摸牌）", "示意或自造（正式包 `mapping: null`，仅在测试包 `decision34_closure` 注册简化版）"),
                    ("mbcmfangzhu", "放逐（道心 >= 75 解锁：受到伤害后令一名其他角色摸牌并翻面或弃牌失去体力）", "示意或自造（正式包 `mapping: null`，仅在测试包 `decision34_closure` 注册简化版）"),
                    ("mbjuejin", "绝进（道心 >= 99 解锁限定技：调整全场护甲并将牌堆/手牌杀闪桃酒移除，获得向死技能）", "示意或自造（正式包 `mapping: null`，仅在测试包 `decision34_closure` 注册简化版）"),
                ]:
                    if l29_sk not in seen_skills:
                        seen_skills.add(l29_sk)
                        skills.append({
                            "id": l29_sk,
                            "ref": f"`{rel}:1`",
                            "summary": l29_desc,
                            "target": "原生规则 / `longxue-upstream-liuli`",
                            "status": l29_st,
                        })
                if "_CMeffects" not in seen_skills:
                    seen_skills.add("_CMeffects")
                    ui_hooks.append({
                        "id": "_CMeffects",
                        "ref": f"`{rel}:1`",
                        "summary": "圣剑显示与阶段切换特效钩子（`lib.skill._CMeffects`）",
                        "target": "原生前端组件",
                        "status": "缺失（圣剑 DOM/Spine 特效未在原生前端完整复刻）",
                    })
                if "shengjian" not in seen_configs:
                    seen_configs.add("shengjian")
                    configs.append({
                        "id": "shengjian",
                        "ref": f"`{rel}:1`",
                        "summary": "圣剑显示开关（`config.shengjian.init = true`）",
                        "target": "原生规则（`longxue.rs`）",
                        "status": "忠实实现",
                    })

        # Object-style character files (e.g. 3D精选/character/character.js, 英雄杀/character/character.js, 杀海拾遗/character/*/character.js)
        if fname in ("character.js", "extension_character.js"):
            _extract_file_top_keys_as_generals(sid, rel, lines, seen_generals, generals, trans, pkg_generals)

        # Standalone card files (card.js, extension_card.js)
        if fname in ("card.js", "extension_card.js"):
            _extract_file_top_keys_as_cards(sid, rel, lines, seen_cards, cards, trans, pkg_cards, pkg_deferred)

        # Standalone skill files (skill.js, extension_skill.js, extension_buff.js, markskill.js)
        if fname in ("skill.js", "extension_skill.js", "extension_buff.js", "markskill.js"):
            _extract_file_top_keys_as_skills(sid, rel, lines, seen_skills, skills, ui_hooks, trans, pkg_skills)

        # Structured sections inside extension.js / content.js / config.js / season files
        if fname in ("extension.js", "extension.readable.js", "config.js", "precontent.js", "content.js", "extension_season.js", "event_universal.js", "extension_servant.js"):
            _extract_structured_blocks(
                sid, rel, fname, lines,
                seen_generals, generals,
                seen_skills, skills, ui_hooks,
                seen_cards, cards,
                seen_configs, configs,
                modes, trans,
                pkg_generals, pkg_skills, pkg_cards, pkg_deferred,
            )

    for cf in css_files:
        rel = rel_to_workspace(cf)
        clines = read_lines_safe(cf)
        surfaces.append({
            "id": cf.name,
            "ref": f"`{rel}:1-{len(clines)}`",
            "summary": f"CSS 样式表 `{cf.name}`（{len(clines)} 行）",
            "target": "原生前端组件",
            "status": "示意或自造（仅抽屉摘要样式，未完整复刻上游 DOM/CSS 布局）" if sid in ("L07", "L08", "L09", "L10", "L11", "L12", "L13", "L14", "L20", "L24", "L26", "L27", "L29") else "忠实实现",
        })

    return {
        "generals": generals,
        "skills": skills,
        "cards": cards,
        "modes": modes,
        "configs": configs,
        "ui_hooks": ui_hooks,
        "surfaces": surfaces,
    }


def _classify_general(
    sid: str,
    cid: str,
    ref: str,
    sex: str,
    grp: str,
    hp: str,
    sk_list: list[str],
    trans: dict[str, str],
    pkg_generals: dict[str, dict],
) -> dict:
    zh = trans.get(cid, cid)
    sk_str = ", ".join(sk_list[:6]) + ("…" if len(sk_list) > 6 else "")
    summary = clean_md(f"{zh}（势力={grp}, 性别={sex}, 体力={hp}, 技能=[{sk_str}]）")
    if cid in pkg_generals:
        g_skills = pkg_generals[cid]
        unmapped = [k for k, v in g_skills.items() if not v]
        if not unmapped:
            status = "忠实实现"
        else:
            status = f"缺失（正式包已登记武将，但技能 {','.join(unmapped)} 为 mapping: null）"
    elif (
        (sid == "U01" and cid == "ddd_zhangmiao")
        or (sid == "L19" and cid in ("jiangshifz", "jiangshinj"))
        or (sid == "L28" and cid in ("th_guanning", "th_majun", "th_shen_zhouyu"))
    ):
        status = "忠实实现（武将与自带技能已按上游语义实现并接入正式包产品路径）"
    elif sid in ("U01", "U05", "L25", "L28"):
        status = "示意或自造（武将已接入正式包产品路径，但部分自带技能尚未逐个完整实现）"
    elif sid == "U08":
        status = "缺失（正式包 hero-kill-upstream-noname 仅含卡牌，未登记 35 名武将）"
    elif sid == "L11":
        status = "缺失（太虚幻境敌方/随从武将未迁入正式包）"
    else:
        status = "缺失"
    return {
        "id": cid,
        "ref": ref,
        "summary": summary,
        "target": "原生规则 / `rule-vm` 包",
        "status": status,
    }


def _classify_card(
    sid: str,
    cd: str,
    ref: str,
    lines: list[str],
    idx: int,
    trans: dict[str, str],
    pkg_cards: dict[str, str],
    pkg_deferred: dict[str, str],
) -> dict:
    summary = summarize_snippet(lines, idx, trans, cd)
    if cd in pkg_cards or cd in (
        "th_taipingyaoshu",
        "zhengsu_leijin",
        "zhengsu_mingzhi",
        "zhengsu_bianzhen",
        "mtg_duzhao",
        "mtg_cangbaohaiwan",
        "pozhenjue",
    ):
        status = "忠实实现"
    elif cd in pkg_deferred:
        if cd in ("diaobingqianjiang", "shichangshi"):
            status = "示意或自造（已接入正式包产品路径，但仍为简化结算分支）"
        else:
            status = f"缺失（正式包列为 deferred：{pkg_deferred[cd][:40]}）"
    else:
        status = "缺失"
    return {
        "id": cd,
        "ref": ref,
        "summary": summary,
        "target": "原生规则 / `rule-vm` 包",
        "status": status,
    }


def _classify_skill_or_hook(
    sid: str,
    sk: str,
    ref: str,
    lines: list[str],
    idx: int,
    trans: dict[str, str],
    pkg_skills: dict[str, str | None],
) -> dict:
    summary = summarize_snippet(lines, idx, trans, sk)
    window = " ".join(lines[idx : min(len(lines), idx + 22)])
    is_ui_hook = (
        sid in UI_ONLY_SOURCES
        or (sk.startswith("_") and sid in ("U04", "L05", "L08", "L09", "L10", "L12", "L14", "L20", "L24", "L27", "L28"))
        or any(k in window for k in ("decadeUI", "playAudio", "setBackgroundImage", "ui.create"))
    )
    if is_ui_hook and sk not in pkg_skills:
        target = "原生前端组件"
        if sid in ("L01", "L02", "L15", "L16", "L17", "L18", "L22", "L23", "L26"):
            status = "忠实实现（已在 content-native-loader 对应模块收口）"
        elif sid == "L09" and sk in ("_qhlyChangeSkin", "_qhly_randskin", "_qhly_autoc"):
            status = "忠实实现（已在 `content-native-loader::qhly` 实现换肤决策）"
        elif sid == "L08" and sk in L08_DECADE_SKILLS_SET:
            status = "示意或自造（已清除摸牌/加甲规则副作用，仅递增 `_ui_fx_seq`；完整 DOM/CSS 特效待在前端复刻）"
        elif sid == "L14" and sk in L14_SHOUSHA_SKILLS_SET:
            status = "示意或自造（已清除摸牌/回血规则副作用，仅递增 `_l14_ui_fx_seq`；完整手杀 UI 钩子待在前端复刻）"
        elif sid == "L24" and sk in L24_SKIN_SKILLS_SET:
            status = "示意或自造（已清除摸牌规则副作用，仅递增 `_l24_skin_fx_seq`；Spine 4.0 净室播放器已落地，其余钩子待补完）"
        elif sid == "L20" and sk in L20_SKILLS_SET:
            status = "示意或自造（已清除默认摸 1 副作用；完整标记特效钩子待在前端复刻）"
        elif sid == "L27" and sk == "_chooseCharacterTwo":
            status = "示意或自造（已清除摸 2 张牌规则副作用；双将选将 UI 钩子待在前端完整复刻）"
        else:
            status = "缺失（前端 UI 钩子未在原生前端完整复刻）"
        return {
            "id": sk,
            "ref": ref,
            "summary": summary,
            "target": target,
            "status": status,
            "is_hook": True,
        }

    target = "原生规则 / `rule-vm` 包"
    if sk in pkg_skills and pkg_skills[sk]:
        if sk == "kuwu_skill" and sid == "U06":
            status = "忠实实现（正式包已映射 `host:sha_damage_target_discard_or_lose_hp` 且接入 `huanle_kuwu_interactive`）"
        else:
            status = f"忠实实现（正式包已映射 `{pkg_skills[sk]}`）"
    elif (
        (sid == "U01" and sk in ("dddxiaxing", "dddyeshen", "dddqiaoduan", "dddqianlong", "dddqice", "dddjungui", "dddshilie", "ddddiedang"))
        or (sid == "U08" and sk == "menshen2")
        or (sid == "L19" and (sk in ("baozou", "ganran", "shishi", "xunmeng", "zaibian") or sk.startswith("chanceIdentity") or sk.startswith("_wujinmoshi")))
        or (sid == "L28" and sk in ("th_dunshi", "th_jingxie", "th_qiaosi", "th_yeyan", "th_taipingyaoshu"))
    ):
        status = "忠实实现（已按上游语义实现并接入正式包产品路径）"
    elif sid in ("L01", "L21", "L22") and sid in PORTABLE_POLICY_SOURCES:
        status = f"忠实实现（已在 `{PORTABLE_POLICY_SOURCES[sid]}` 实现）"
    elif (
        sk in DECISION34_SKILL_SET
        or (sid == "U01" and sk.startswith("ddd"))
        or (sid == "U05" and sk in U05_D34_SKILLS_SET)
        or (sid == "L19" and sk in L19_SKILLS_SET)
        or (sid == "L20" and sk in L20_SKILLS_SET)
        or (sid == "L25" and sk in L25_SKILLS_SET)
        or (sid == "L27" and sk in L27_SKILLS_SET)
        or (sid == "L28" and sk.startswith("th_"))
    ):
        status = "示意或自造（已清除默认摸 1 兜底并接入正式包，但完整上游分支待逐项补齐）"
    elif sk in pkg_skills and not pkg_skills[sk]:
        status = "缺失（正式包中 `mapping: null`）"
    else:
        status = "缺失"
    return {
        "id": sk,
        "ref": ref,
        "summary": summary,
        "target": target,
        "status": status,
        "is_hook": False,
    }


L08_DECADE_SKILLS_SET = {
    "_changeEquipExt", "_ui_auto_cheat", "_kill_framebg", "_rarity", "_tiesuo_ying",
    "_usecard", "_decadeUI_usecardBegin", "_discard", "_decadeUI_dieKillEffect",
}
L14_SHOUSHA_SKILLS_SET = {
    "_dieNoTuoguan", "_menuCheat", "_jindutiaoO", "_jindutiaoA", "_jindutiaoMB",
    "_jindutiaoKS", "_jindutiao_close", "_chupaiA", "_chupaiB", "_chupaiC",
    "_chupaiD", "_chupaiE", "_chupaiF", "_chupaiG", "_chupaiH",
    "_chupaiM", "_chupaiN", "_chupaiO", "_chupaiP", "_chupaiI",
    "_chupaiJ", "_chupaiK", "_chupaiL", "_chupaiMX", "_chupaiNX",
    "_skilltip_closeB", "_jd_ddxyA", "_jd_ddxyB", "_jd_ddxyC", "_jd_ddxyD",
    "_jd_ddxyF", "_jd_ddxyAll", "_jd_shiyongshanD", "_jd_ddxyE", "_jd_wjsw",
    "_jd_swxs", "_jd_dfsk", "_jd_hhks", "_jd_pdjd", "_jd_mpjd",
    "_jd_cpjd", "_jd_qpjd", "_jd_hhjs", "_jd_phaseAfter", "_jindutiao",
    "_jindutiaopl", "_wmkzSayChange", "_shuohuaSayWordphase",
}
L19_SKILLS_SET = {
    "baozou", "shishi", "xunmeng", "zaibian", "ganran", "ganran2",
    "_jisuangailv", "_tuizhi", "_tuizhi2", "_jiangshi", "_jiangshi2",
    "_jiangshi3", "_jiangshi4", "_jiangshiTx", "_jiangshiTx2", "_jiangshiTx3",
    "_huzhu", "_gainSkill", "_keepideskill", "_levelskill", "_expskill",
    "_exp2skill", "_useskill", "_killskill", "_relifeskill", "_yuanqiskill",
    "_zhongzhubuff", "_jingchengadddie", "_zhongfandieskill", "_tamodeskill",
    "_zhudieskill", "_qunxiongzhengbaskill", "_guanmodeskill", "_guandieskill",
    "_guixinmodeskill", "_huanhaiskill", "_seadreammodeskill", "_dietime", "_anshaskill",
}
L20_SKILLS_SET = {
    "_xigua_feiyanbahu_", "_shiqihujia_", "zhengsu", "zhenyi_spade", "xinhongyan",
    "_guozhanChooseControlCHG", "diaohulishan", "_guozhan_marks", "xianqu_mark",
    "zhulianbihe_mark", "yinyang_mark", "yexinjia_mark", "_zhulianbihe_mark_tao",
    "_yinyang_mark_add", "_guozhan_diebiaoji_", "_guozhan_duiweibiaoji_gua_",
    "_guozhan_duiweibiaoji_gua_gua_", "_aozhan_judge", "_guozhanhalfchghp",
    "_xigua_xueliangfanbei_", "_xigua_mopaifanbei_", "_xigua_zhuwanzhongji_",
    "_shoushajisha_", "_shoushajisha_delete_", "_diankuangtulu_gua", "_wanjunqushou_gua",
    "_o_o_recovertrigger", "_o_o_miaoshouhuichun", "_o_o_yishugaochao", "_losehp_",
    "_mianshang_", "_player_baojiA", "_player_baojiB", "_player_baojiC", "_player_baojiD",
    "_player_baojiE", "_player_baojiF", "_player_baojiG", "_player_baojiH", "_jiubuff_",
    "_jiubuff_xiaoshi_", "_player_game_guagua", "_guozhan_quanjubiaoji_guagua",
    "_shiyongkapaitexiao_", "_shiyongkapaitexiao_xi_gua_", "_shanghaipaishiyongtexiao_",
    "_player_daojianfu_xxgg_fuzi_", "_binglintu", "_dysp", "xinfu_langxi",
    "tspowei", "qingyu", "mibei", "fz_new_rewusheng", "fz_gzpaoxiao",
    "fz_new_longdan", "fz_new_tieji", "fz_liegong", "fz_xinkuanggu", "rekuanshi",
    "kuanshi", "kuanshi2", "shouxi", "rechuhai", "xinfu_lingren",
    "sbtieji", "sbduanliang", "shouye", "mbmowang", "wengua",
    "wengua5", "zuoxing", "kurou", "_chongzhu", "ms_longmark",
}
L24_SKIN_SKILLS_SET = {
    "_tsx", "_gj", "_ts", "_playAudioToQueue", "_checkDcdChuChang",
    "_checkDcdShan", "_fix_yh", "_fix_phase_yh", "_pfqh_check_roundCount", "_check_die_yh",
    "_pfqh_reapply_saved_params", "_pfqh_reapply_on_init", "_pfqh_check_hp_change",
    "_pfqh_check_damage_times", "_pfqh_check_damage", "_pfqh_check_jisha",
    "_pfqh_check_changeGroup", "_qhlyChangeGroupSkin", "_pfqh_check_phaseBegin",
    "_pfqh_check_phaseEnd", "_qhly_autoc", "_qhly_addButton", "_skin_switch_hook",
}
L25_SKILLS_SET = {
    "g_du", "_player_xxgg_zhishixian_xxxxx_ggg", "_player_xxgg_zhishixian2_xxxxx_ggg",
    "_player_xxgg_zhishixian3_xxxxx_ggg", "_jszsx", "_zuanattacklinecheck",
    "_strengthencardskill", "_strengthenstandardskill", "_ssrnohuahensskill",
    "_ssrchangehpskill", "_diycharacterskill", "_emojiRebate1", "_emojiRebate2",
    "_zuan_damageSendEgg", "_zuan_sendWine", "_zuan_damageSendFlower", "_zuan_killSendFlower",
    "_extraphaseskill", "_extradieskill", "_extraturnoverskill", "_extradamageskill",
    "_extradlosehpskill", "_extraloseskill", "_extragainskill", "_extraloseCardskill",
    "_extracompareskill", "_extrauseskill", "_extrajudgeskill", "_extralinkskill",
    "_comparesayskill", "_equipuseskill", "_damsayskill", "_saveskill", "_dieskill",
    "_shanskill", "_useCardsayskill", "_phasebeginskill", "_phaseendskill", "_judgeskill",
    "_linkskill", "_zuanwashhandcard", "_newbiankuang", "_clearBackground", "_newBackground",
    "_newBackgroundUse", "_newBackgroundAtart", "_zuanboss", "_zuanRoundTX", "_newZuanPhaseTX",
    "_zuanphaseTX", "_helasisyskill", "_keepideskill", "_zuanidentityskill",
    "_zuanidentitychgneiskill", "_zuanidentitydieafterskill", "_zuanidentityshowskill",
    "_zuanidentitydyingskill", "_zuanidentitytodieskill", "_zuanidentitydamskill",
    "_zuanidentityrecskill", "_zuanidentityuseskill", "_yuanqiskill", "_anyeskill",
    "_taohuaskill", "_lunjianskill", "_jianghunskill", "_dietime", "_anshaskill",
    "_huanhaiskill", "_seadreammodeskill", "_thunderjudgemodeskill", "_thundermodeskill",
    "_guixinmodeskill", "_guanmodeskill", "_guandieskill", "_qunxiongzhengbaskill",
    "_zhongzhubuff", "_jingchengadddie", "_zhongfandieskill", "_tamodeskill",
    "_zhudieskill", "_relifeskill", "_levelskill", "_expskill", "_exp2skill",
    "_useskill", "_killskill", "_zuanSTARTsayskill", "_zuanUSECARDsayskill",
    "_zuanCHANGEHPsayskill", "_zuansaidskill", "_recoversay", "_damagesay", "_diesay",
    "_tosay", "_fromsay", "_selfsay", "_startgamesay", "_haoyunlai", "_basicjuedgeskill",
    "_goodluckjuedgeskill", "_bingleAddJudge", "_bingleLoseJudge", "_bingleBigScreen",
    "_bingleDying", "_binglediecls", "_bingletexiaoshandian", "_bingletexiaohuoshan",
    "_bingletexiaohongshui", "_bingletexiaofulei", "_bingletexiaole", "_bingletexiaobingliang",
    "_bingletexiao", "_thundertexiao", "_clearthunder", "_thundertexiao2",
    "_bingletexiaochangehp", "_bingletexiao1", "_bingletexiao2", "_diyskill",
    "wumou", "new_guixin",
}
L27_SKILLS_SET = {
    "baonvezhi", "baonvezhi_mark", "bolan", "clandianzhan", "dcxiaoxi",
    "dinghan", "drlt_duorui", "jinghe", "jishe", "kuangcai",
    "new_meibu", "new_zhixi", "new_zhixi_recover", "pro_drlt_duorui", "qimou",
    "quanji", "regushe", "rehuashen", "rehuashen_mark", "reqimou",
    "rexinsheng", "sbjianxiong", "sbtongye", "sbzhaxiang", "shouxi",
    "spyanjiao", "taoluan", "twfengji", "twharvestinori", "twxiafeng",
    "twxingzhui", "twzhouhu", "twzhouzu", "twzuhuo", "xiefang",
    "xinfu_xionghuo", "xingchong",
}
U05_D34_SKILLS_SET = {
    "duijue", "tanlin", "ycndiyjianxiong", "xinfu_jingxie", "ycnxinfu_qiaosi",
    "lichang", "feiying", "maoding", "bingqiang", "lianpo",
    "zhanlong", "huoyu", "mianzhen", "aqianghua", "shanguang",
    "tiandan", "liudan", "shoujia", "zhongdun", "bfengshi",
    "woliu", "dianji", "qinru", "maichong", "yihun",
    "shouge", "tuijin", "zihui", "chongzhuang", "shanxian", "shanhui",
    "feizhua", "leiyu", "qixia", "jianzhen", "xjzongyu",
    "huxi", "qijian", "tuoqiao", "duxinshu", "tianjian",
    "xfenxin", "fenxing", "feixia", "lingdi", "qiongguang",
    "yanzhan", "longxi", "tianwu", "danqing", "dujiang",
    "xiaoyue", "tanhua", "xuanning", "ywuhun", "yujia", "yanshi",
}


def _extract_file_top_keys_as_generals(
    sid: str,
    rel: str,
    lines: list[str],
    seen: set[str],
    out: list[dict],
    trans: dict[str, str],
    pkg_generals: dict[str, dict],
) -> None:
    re_top = re.compile(r"""^(\t+| {2,16})["']?([A-Za-z0-9_]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        m = re_top.match(line)
        if not m:
            continue
        cid = m.group(2)
        if cid in seen or cid in RESERVED_SKILL_KEYS:
            continue
        window = "\n".join(lines[idx : min(len(lines), idx + 12)])
        if "sex:" in window or "group:" in window or "hp:" in window:
            seen.add(cid)
            sex = re.search(r"""sex\s*:\s*["']([^"']+)["']""", window)
            grp = re.search(r"""group\s*:\s*["']([^"']+)["']""", window)
            hp = re.search(r"""hp\s*:\s*([0-9/'"A-Za-z_]+)""", window)
            sk = re.search(r"""skills\s*:\s*\[([^\]]*)\]""", window)
            sk_list = (
                [s.strip().strip("'\"") for s in sk.group(1).split(",") if s.strip().strip("'\"")]
                if sk
                else []
            )
            out.append(
                _classify_general(
                    sid,
                    cid,
                    f"`{rel}:{idx + 1}`",
                    sex.group(1) if sex else "?",
                    grp.group(1) if grp else "?",
                    hp.group(1) if hp else "?",
                    sk_list,
                    trans,
                    pkg_generals,
                )
            )


def _extract_file_top_keys_as_cards(
    sid: str,
    rel: str,
    lines: list[str],
    seen: set[str],
    out: list[dict],
    trans: dict[str, str],
    pkg_cards: dict[str, str],
    pkg_deferred: dict[str, str],
) -> None:
    re_top = re.compile(r"""^(\t+| {2,16})["']?([A-Za-z0-9_]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        m = re_top.match(line)
        if not m:
            continue
        cd = m.group(2)
        if cd in seen or cd in RESERVED_SKILL_KEYS:
            continue
        window = "\n".join(lines[idx : min(len(lines), idx + 12)])
        if any(k in window for k in ("type:", "subtype:", "fullskin:", "enable:", "vanish:", "cardimage:")):
            seen.add(cd)
            out.append(_classify_card(sid, cd, f"`{rel}:{idx + 1}`", lines, idx, trans, pkg_cards, pkg_deferred))


def _extract_file_top_keys_as_skills(
    sid: str,
    rel: str,
    lines: list[str],
    seen: set[str],
    skills: list[dict],
    ui_hooks: list[dict],
    trans: dict[str, str],
    pkg_skills: dict[str, str | None],
) -> None:
    re_top = re.compile(r"""^(\t| {2,4})["']?([A-Za-z0-9_$]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        m = re_top.match(line)
        if not m:
            continue
        sk = m.group(2)
        if sk in seen or sk in RESERVED_SKILL_KEYS:
            continue
        seen.add(sk)
        item = _classify_skill_or_hook(sid, sk, f"`{rel}:{idx + 1}`", lines, idx, trans, pkg_skills)
        if item["is_hook"]:
            ui_hooks.append(item)
        else:
            skills.append(item)


def _extract_structured_blocks(
    sid: str,
    rel: str,
    fname: str,
    lines: list[str],
    seen_generals: set[str],
    generals: list[dict],
    seen_skills: set[str],
    skills: list[dict],
    ui_hooks: list[dict],
    seen_cards: set[str],
    cards: list[dict],
    seen_configs: set[str],
    configs: list[dict],
    modes: list[dict],
    trans: dict[str, str],
    pkg_generals: dict[str, dict],
    pkg_skills: dict[str, str | None],
    pkg_cards: dict[str, str],
    pkg_deferred: dict[str, str],
) -> None:
    re_key_obj = re.compile(r"""^(\s*)["']?([A-Za-z0-9_$\u4e00-\u9fa5]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        lineno = idx + 1
        # Season / event / servant nodes in L11
        if fname in ("extension_season.js", "event_universal.js", "extension_servant.js"):
            m_k = re_key_obj.match(line)
            if m_k and len(m_k.group(1)) in (1, 2, 4, 8):
                kid = m_k.group(2)
                if kid not in RESERVED_SKILL_KEYS and not kid.startswith("//"):
                    window = " ".join(lines[idx : min(len(lines), idx + 12)])
                    m_name = re.search(r"""name\s*:\s*["']([^"']+)["']""", window)
                    zh = m_name.group(1) if m_name else trans.get(kid, kid)
                    modes.append({
                        "id": kid,
                        "ref": f"`{rel}:{lineno}`",
                        "summary": clean_md(f"{fname} 节点 `{kid}`（{zh}）"),
                        "target": "原生规则（`taixu.rs`）+ 原生前端组件",
                        "status": "缺失（仅 `TaixuRunState` 骨架，赛季/事件/侍灵未完整迁入正式包）",
                    })
            continue

        m = re_key_obj.match(line)
        if not m:
            continue
        indent, key = m.group(1), m.group(2)
        if key in RESERVED_SKILL_KEYS:
            continue
        window = "\n".join(lines[idx : min(len(lines), idx + 16)])

        # Config entry detection: has `name` and (`init` or `intro` or `onclick` or `item`) inside a small block
        has_cfg_name = re.search(r"""["']?name["']?\s*:""", window) is not None
        has_cfg_meta = re.search(r"""["']?(?:init|intro|onclick|item)["']?\s*:""", window) is not None
        if has_cfg_name and has_cfg_meta and "trigger:" not in window and "content:" not in window:
            if key not in seen_configs:
                seen_configs.add(key)
                m_n = re.search(r"""["']?name["']?\s*:\s*["']([^"']+)["']""", window)
                m_i = re.search(r"""["']?init["']?\s*:\s*([^,\n]+)""", window)
                m_intro = re.search(r"""["']?intro["']?\s*:\s*["']([^"']+)["']""", window)
                zh = clean_md(m_n.group(1), 40) if m_n else key
                init_v = clean_md(m_i.group(1), 25) if m_i else "—"
                intro_v = clean_md(m_intro.group(1), 50) if m_intro else ""
                summary = clean_md(f"{zh}（默认={init_v}）{('：' + intro_v) if intro_v else ''}")
                target = (
                    f"原生规则（`{PORTABLE_POLICY_SOURCES[sid]}`）"
                    if sid in PORTABLE_POLICY_SOURCES
                    else "原生规则 / 配置 schema"
                )
                status = "忠实实现" if sid in PORTABLE_POLICY_SOURCES or sid in ("U01", "U05", "U06", "U07", "U08", "L20", "L27") else "缺失"
                configs.append({
                    "id": key,
                    "ref": f"`{rel}:{lineno}`",
                    "summary": summary,
                    "target": target,
                    "status": status,
                })
            continue

        # Card object inside extension.js (has `fullskin:` or `subtype:` or (`type:` and `enable:` and not `trigger:`))
        if (
            ("fullskin:" in window or "subtype:" in window or ("type:" in window and "enable:" in window))
            and "trigger:" not in window
            and "subSkill:" not in window
        ):
            if key not in seen_cards:
                seen_cards.add(key)
                cards.append(_classify_card(sid, key, f"`{rel}:{lineno}`", lines, idx, trans, pkg_cards, pkg_deferred))
            continue

        # Skill / hook object inside extension.js (has `trigger:`, `enable:`, `mod:`, `content:`, `viewAs:`, `filter:`, `mark:`, `init:` with `player`)
        if any(
            kw in window
            for kw in ("trigger:", "content:", "viewAs:", "filterCard:", "filterTarget:", "mod:", "subSkill:", "marktext:", "enable:")
        ):
            if len(indent.expandtabs(4)) <= 16 and key not in seen_skills:
                seen_skills.add(key)
                item = _classify_skill_or_hook(sid, key, f"`{rel}:{lineno}`", lines, idx, trans, pkg_skills)
                if item["is_hook"]:
                    ui_hooks.append(item)
                else:
                    skills.append(item)


def count_statuses(inv: dict[str, list[dict]], d34_rows: list[tuple]) -> dict[str, int]:
    counts = {"忠实实现": 0, "示意或自造": 0, "缺失": 0}
    for cat in ("generals", "skills", "cards", "modes", "configs", "ui_hooks", "surfaces"):
        for item in inv.get(cat, []):
            st = item["status"]
            if st.startswith("忠实实现"):
                counts["忠实实现"] += 1
            elif st.startswith("示意或自造"):
                counts["示意或自造"] += 1
            else:
                counts["缺失"] += 1
    return counts


def render_decision34_section(
    sid: str,
    name: str,
    inv: dict[str, list[dict]],
) -> tuple[str, list[tuple[str, str]]]:
    """Return (main_markdown_block, extra_part_files) keeping every file <= 1,850 lines."""
    d34_rows = DECISION34_AUDIT.get(sid, [])
    counts = count_statuses(inv, d34_rows)
    total = sum(counts.values())

    lines: list[str] = [
        "## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-09）",
        "",
        f"- **统计汇总**：上游实体合计 **{total}** 项（武将 {len(inv['generals'])} / 技能 {len(inv['skills'])} / 卡牌 {len(inv['cards'])} / 模式与战役节点 {len(inv['modes'])} / 配置项 {len(inv['configs'])} / 界面钩子 {len(inv['ui_hooks'])} / 渲染面 {len(inv['surfaces'])}）",
        f"- **状态分布**：`忠实实现` **{counts['忠实实现']}** 项 · `示意或自造` **{counts['示意或自造']}** 项 · `缺失` **{counts['缺失']}** 项",
        "",
        "### 7.1 现有 `decision34*` / 前端示意代码逐条归类",
        "",
    ]
    if d34_rows:
        lines.extend([
            "| 现有实现项 | 原生文件:行号 | 上游对照 (`file:line`) | 原生落点 | 状态与处置结论 |",
            "|---|---|---|---|---|",
        ])
        for item_name, native_ref, up_ref, target, disp in d34_rows:
            lines.append(f"| {item_name} | {native_ref} | {up_ref} | {target} | {disp} |")
    else:
        lines.append("- 本来源未被 `decision34_closure.rs` / `decision34_pack9.rs` 注入自造武将或默认摸 1 技能。")
    lines.append("")

    categories = [
        ("7.2", "武将（Generals）", inv["generals"]),
        ("7.3", "技能（Skills）", inv["skills"]),
        ("7.4", "卡牌（Cards）", inv["cards"]),
        ("7.5", "模式 / 战役节点（Modes & Campaign Nodes）", inv["modes"]),
        ("7.6", "配置项（Config Items）", inv["configs"]),
        ("7.7", "界面钩子（UI Hooks）", inv["ui_hooks"]),
        ("7.8", "渲染面（Rendering Surfaces）", inv["surfaces"]),
    ]

    all_item_rows: list[tuple[str, str, dict]] = []
    for sec_no, sec_title, items in categories:
        for it in items:
            all_item_rows.append((sec_no, sec_title, it))

    # If all rows fit comfortably in the main file (< 1,500 lines), put them directly in the main file
    if len(all_item_rows) <= 1350:
        for sec_no, sec_title, items in categories:
            lines.append(f"### {sec_no} {sec_title}（{len(items)} 项）")
            lines.append("")
            if not items:
                lines.append("- （上游无此类别实体）")
                lines.append("")
                continue
            lines.append("| 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |")
            lines.append("|---|---|---|---|---|")
            for it in items:
                lines.append(
                    f"| `{it['id']}` | {it['ref']} | {it['summary']} | {it['target']} | {it['status']} |"
                )
            lines.append("")
        return "\n".join(lines), []

    # Otherwise, split across main file + companion -partN.md files so every file is <= 1,850 lines
    slug = name.replace(" ", "-")
    chunk_size = 1500
    extra_files: list[tuple[str, str]] = []
    first_chunk = all_item_rows[:1100]
    remaining = all_item_rows[1100:]

    part_links = []
    for p_idx in range(0, len(remaining), chunk_size):
        part_num = (p_idx // chunk_size) + 2
        part_fname = f"{sid}-{slug}-part{part_num}.md"
        part_links.append(f"[{part_fname}](./{part_fname})")

    lines.append(
        f"> 注：本来源实体共 {len(all_item_rows)} 项，为遵守单文件 ≤2,000 行门禁，第 1–{len(first_chunk)} 项列于本页，后续分册见：{'、'.join(part_links)}。"
    )
    lines.append("")
    lines.append("### 7.2–7.8 逐项清单（主表）")
    lines.append("")
    lines.append("| 类别 | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |")
    lines.append("|---|---|---|---|---|---|")
    for sec_no, sec_title, it in first_chunk:
        short_cat = sec_title.split("（")[0]
        lines.append(
            f"| {short_cat} | `{it['id']}` | {it['ref']} | {it['summary']} | {it['target']} | {it['status']} |"
        )
    lines.append("")

    for p_idx in range(0, len(remaining), chunk_size):
        part_num = (p_idx // chunk_size) + 2
        part_fname = f"{sid}-{slug}-part{part_num}.md"
        chunk = remaining[p_idx : p_idx + chunk_size]
        start_no = 1100 + p_idx + 1
        end_no = 1100 + p_idx + len(chunk)
        p_lines = [
            f"# G0 · {sid} {name}（裁决 34 逐项清单 · 分册 {part_num}：第 {start_no}–{end_no} 项）",
            "",
            f"> 主文档：[{sid}-{slug}.md](./{sid}-{slug}.md) · 台账：[upstream-extension-reconstruction.md](../upstream-extension-reconstruction.md)",
            "",
            "| 类别 | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |",
            "|---|---|---|---|---|---|",
        ]
        for sec_no, sec_title, it in chunk:
            short_cat = sec_title.split("（")[0]
            p_lines.append(
                f"| {short_cat} | `{it['id']}` | {it['ref']} | {it['summary']} | {it['target']} | {it['status']} |"
            )
        p_lines.append("")
        extra_files.append((part_fname, "\n".join(p_lines)))

    return "\n".join(lines), extra_files
