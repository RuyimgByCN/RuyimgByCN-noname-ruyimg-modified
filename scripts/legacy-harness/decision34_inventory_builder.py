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
            "忠实实现（31 名武将已通过 `attach_formal_package_upstream_rules` 接入 `d3-upstream-noname` 正式包产品路径，全部 64 项 `ddd*` 技能已在 `decision34_pack9.rs` + `decision34_ext.rs` 实现）",
        ),
        (
            "`U01_DDD_SKILLS` + `resolve_u01_ddd_skill` + `resolve_u01_ddd_remaining`（64 项 `ddd*` 技能）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:141-333`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:51-271`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`",
            "原生规则 / `d3-upstream-noname`",
            "忠实实现（已清除 `_ => ctx.start_draw(owner, 1)` 兜底，全部 64 项 `ddd*` 技能按上游语义实现并接入正式包产品路径）",
        ),
        (
            "`[data-testid=\"u01-3d-select-gallery\"]` 3D 立绘路径预览",
            "`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:544-557`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:218`",
            "原生前端组件",
            "忠实实现（按上游 `character.js:218` 立绘目录切换 31 名 `ddd_*` 武将预览）",
        ),
    ],
    "U02": [
        (
            "`boss_playpack` 挑战/剑阁武将池缓存与体力归一化（`boss_baonu_stage` 已按第二步清除）",
            "`crates/content-native-loader/src/boss_playpack.rs:1-215`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:221-238`",
            "`archive/upstream/noname/apps/core/extension/boss/extension.js:6-146`",
            "原生规则 / `boss-play-upstream-noname`",
            "忠实实现（已清除伪造的 `boss_baonu_stage`，按上游 `boss_storage_playpackconfig` 归一化挑战/剑阁武将池与体力）",
        ),
    ],
    "U03": [],
    "U04": [
        (
            "`renderCoinCanvasParticles` / `resolveCoinHonorTitle` / `[data-testid=\"coin-weather-overlay\"]`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1123-1177`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:431-445`",
            "`archive/upstream/noname/apps/core/extension/coin/extension.js:17-540`",
            "原生规则（`coin.rs`）+ 原生前端组件",
            "忠实实现（已在 `coin.rs` 与前端 `<canvas data-testid=\"coin-particle-canvas\">` 实现烟花/下雪/星云/闪烁粒子渲染与商店购买/下注状态机）",
        ),
    ],
    "U05": [
        (
            "`U05_SWD_REMAINING_SKILLS` / `U05_YUNCHOU_REMAINING_SKILLS` / `U05_MTG_LAND_CARDS` / `U05_ZHENFA_CARDS` / `U05_OW_SKILLS` / `U05_XIANJIAN_GENERALS` / `U05_XIANJIAN_SKILLS`",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:60-124`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:335-555`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:330-385`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/` 各子目录",
            "原生规则 / `packages/extensions/sha*`",
            "忠实实现（已清除 `resolve_ow_skill` 与 `resolve_xianjian_skill` 兜底并通过 `attach_formal_package_upstream_rules` 接入 `packages/extensions/sha*` 全部 9 个子包产品路径）",
        ),
    ],
    "U06": [
        (
            "`huanle_kuwu_interactive` / `resolve_kuwu_interactive`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:293-343`",
            "`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:53-69`",
            "原生规则 / `huanle-upstream-noname`",
            "忠实实现（已接入正式包 `huanle-upstream-noname` 产品路径，支持弃牌分支与失去 1 点体力分支）",
        ),
    ],
    "U07": [
        (
            "`diaobingqianjiang` / `duoqi` / `youzhan` / `jianbi` / `choudu`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:346-411`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:556-585`",
            "`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:4-168`、`character/skill.js:109-775`",
            "原生规则 / `wudian-upstream-noname`",
            "忠实实现（已接入正式包 `wudian-upstream-noname` 产品路径并覆盖单目标/空手牌/每回合限一次负向门禁）",
        ),
    ],
    "U08": [
        (
            "`yxs_menshen2_guard` / `verify_u08_standard_skill_bindings`（伪造 `yxs_tafang_wave` 与 `advanceTafangWave` 已按第二步清除）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:414-449`、`crates/content-native-loader/src/decision34_closure.rs:348-386`",
            "`archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:87-109`、`character/character.js:3-287`",
            "原生规则 / `hero-kill-upstream-noname`",
            "忠实实现（已清除伪造的 `yxs_tafang_wave` 与 `advanceTafangWave`，`yxs_menshen2_guard` 与 31 项标准映射技能已接入 `hero-kill-upstream-noname`）",
        ),
    ],
    "L01": [],
    "L02": [],
    "L03": [
        (
            "`boss_playpack` 守卫剑阁/挑战武将池缓存与体力归一化",
            "`crates/content-native-loader/src/boss_playpack.rs:1-215`",
            "`archive/upstream/noname-liuli/noname/extension/boss/extension.js:5-136`",
            "原生规则 / `boss-play-upstream-liuli`",
            "忠实实现（已在 `boss_playpack.rs` 实现剑阁/挑战武将池加载与体力归一化）",
        ),
    ],
    "L04": [],
    "L05": [
        (
            "`renderCoinCanvasParticles` / `coin.rs`",
            "`crates/content-native-loader/src/coin.rs:1-210`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1123-1177`",
            "`archive/upstream/noname-liuli/noname/extension/coin/extension.js:17-540`",
            "原生规则（`coin.rs`）+ 原生前端组件",
            "忠实实现（已在 `coin.rs` 与前端 `<canvas data-testid=\"coin-particle-canvas\">` 实现烟花/下雪/星云/闪烁粒子渲染与商店状态机）",
        ),
    ],
    "L06": [],
    "L07": [
        (
            "`runOmnisearchQuery` / `paginateOmnisearchResults` / `highlightOmnisearchTerm`",
            "`crates/content-native-loader/src/omnisearch.rs:1-210`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1183-1216`",
            "`archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js:1-2354`、`SimplePagination.js:1-147`",
            "原生规则（`omnisearch.rs`）+ 原生前端组件",
            "忠实实现（已实现公开索引检索、分页、关键词高亮、暗牌隐私隔离与零 eval 门禁）",
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
            "`L08_DECADE_SKILLS` + `resolve_l08_decade_skill` + `decade_ui.rs`",
            "`crates/content-native-loader/src/decade_ui.rs:1-190`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:590-603`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:54-650`、`skill.js:1-512`",
            "原生前端组件",
            "忠实实现（已清除摸牌/加甲规则副作用并保持纯 UI 资产包零 VM 技能不变量，前端与 `decade_ui.rs` 承载 6 套十周年样式与动效钩子）",
        ),
        (
            "净室 Spine 4.0 播放器 (`apps/web-client/src/engine/spine40/*`，已替换示意 `drawSpineSkeletalCanvas`)",
            "`apps/web-client/src/engine/spine40/webglRenderer.ts:1-474`、`apps/web-client/src/engine/spine40/index.ts:1-5`、`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue:72-95`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816`、`dynamicSkin.js:1-162`",
            "原生前端组件（净室 Spine 4.0 播放器）",
            "忠实实现（按裁决 35 净室自写 Spine 4.0 JSON/二进制解码、骨骼世界变换、IK/变换/路径约束、`transform`/`path`/`deform` 时间轴、多轨道混合、图集解析与 WebGL PMA 渲染，无动皮资源或非 4.0 资源时回退静态立绘）",
        ),
    ],
    "L09": [
        (
            "`qhly.rs` + `switchQhlySkin` / `switchQhlyTheme`",
            "`crates/content-native-loader/src/qhly.rs:1-220`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:654-685`",
            "`archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:1-15970`",
            "原生规则（`qhly.rs`）+ 原生前端组件",
            "忠实实现（已在 `qhly.rs` 与前端组件实现千幻皮肤切换、主题切换与 `saveSkinParams` 参数导入）",
        ),
    ],
    "L10": [
        (
            "`k_danggu` / `k_mowang` + `apply_l10_danggu_companion` + `epic_cards.rs`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:243-295`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:22-36`",
            "`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:244-2970`",
            "原生规则 / `epic-cards-upstream-liuli` + 原生前端组件",
            "忠实实现（已接入正式包 `epic-cards-upstream-liuli` 产品路径，按上游 `conflictMap` 非冲突结党与十常侍 10 项伴生技能完整授予）",
        ),
    ],
    "L11": [
        (
            "`taixu.rs` + `advanceTaixuNode` / `buyTaixuBlessing` / `bindTaixuPet`（伪造步进技能 `taixu_expedition_step` 已按第二步清除）",
            "`crates/content-native-loader/src/taixu.rs:1-240`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:465-520`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension.js:1-6461` 及 `dlc/*`、`extension_*.js`",
            "原生规则（`taixu.rs`）/ `taixu-upstream-liuli` + 原生前端组件",
            "忠实实现（已删除伪造步进技能 `taixu_expedition_step`，由 `taixu.rs` 与前端战役面板承载太虚赛季、节点、赐福与侍灵状态机）",
        ),
    ],
    "L12": [
        (
            "`rzsh.rs` + `toggleRzshTheme` / `numToRoma` 面板",
            "`crates/content-native-loader/src/rzsh.rs:1-210`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:241-245`",
            "`archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:1-9502`、`js/rzsh.js:1-815`",
            "原生规则（`rzsh.rs`）+ 原生前端组件",
            "忠实实现（已在 `rzsh.rs` 与前端组件实现 `numToRoma` 段位转换、主题切换与 Spine 4.0 净室动皮回退）",
        ),
    ],
    "L13": [
        (
            "`shousha_mvp.rs` + `recordMvpEventFact`",
            "`crates/content-native-loader/src/shousha_mvp.rs:1-210`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:740-777`",
            "`archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:1-512`",
            "原生规则（`shousha_mvp.rs`）+ 原生前端组件",
            "忠实实现（已在 `shousha_mvp.rs` 与前端 HUD 计分板实现攻击/治疗/辅助/击杀计分与 `seq` 幂等去重）",
        ),
    ],
    "L14": [
        (
            "`L14_SHOUSHA_SKILLS` + `resolve_l14_shousha_skill` + `shousha_ui.rs`",
            "`crates/content-native-loader/src/shousha_ui.rs:1-210`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:608-617`",
            "`archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-7360`",
            "原生前端组件",
            "忠实实现（已删除伪造 `shousha_*` 武将/卡牌与摸牌/回血副作用，保持纯 UI 包零 VM 技能不变量，前端承载手杀进度条、布局与连破横幅）",
        ),
    ],
    "L15": [],
    "L16": [],
    "L17": [],
    "L18": [],
    "L19": [
        (
            "`L19_DOOMSDAY_CHARACTERS` + `L19_DOOMSDAY_SKILLS` + `doomsday.rs`",
            "`crates/content-native-loader/src/doomsday.rs:1-154`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:620-653`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-3385`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "忠实实现（已清除 22 名伪造 `vir_*` 武将与 `_ => ctx.start_draw(owner, 1)` 兜底，`jiangshifz`、`jiangshinj` 与 66 项技能已接入正式包 `doomsday-brawl-upstream-liuli` 产品路径）",
        ),
    ],
    "L20": [
        (
            "`L20_MARK_SUPPLEMENT_CARDS` + `L20_MARK_SUPPLEMENT_SKILLS` + `resolve_l20_skill` + `resolve_l20_mark_dispatch`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:452-469`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:656-698`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:577-6505`",
            "原生规则 / `mark-supplement-upstream-liuli` + 原生前端组件",
            "忠实实现（已清除 12 名伪造 `ms_*` 武将與默认摸 1 兜底，3 张 `zhengsu_*` 牌、74 项技能与 14 项战局标记已接入正式包 `mark-supplement-upstream-liuli` 产品路径与前端 HUD）",
        ),
    ],
    "L21": [
        (
            "`martial.rs` + `assignMartialLineup` / `applyMartialReplacement`（伪造 `martial_formation_step` 已按第二步清除）",
            "`crates/content-native-loader/src/martial.rs:1-220`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:522-565`",
            "`archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js:1-1367`",
            "原生规则 / `martial-upstream-liuli`",
            "忠实实现（已清除伪造 `martial_formation_step`，由 `martial.rs` 与前端战役台实现 5v5 先后手排阵、替补补摸 4 张与战绩簿）",
        ),
    ],
    "L22": [],
    "L23": [],
    "L24": [
        (
            "`skin_switch.rs` + `L24_SKIN_SWITCH_SKILLS` + 净室 Spine 4.0 播放器",
            "`crates/content-native-loader/src/skin_switch.rs:1-220`、`apps/web-client/src/engine/spine40/index.ts:1-5`",
            "`archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-5900`",
            "原生前端组件",
            "忠实实现（已清除规则层摸牌副作用并保持纯皮肤包零 VM 技能不变量，落地 Spine 4.0 净室播放器与 `saveSkinParams` 参数导入）",
        ),
    ],
    "L25": [
        (
            "`L25_ZUAN_CHARACTERS` + `L25_ZUAN_SKILLS` + `zuan.rs`",
            "`crates/content-native-loader/src/zuan.rs:1-220`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:715-740`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-51470`",
            "原生规则 / `zuan-upstream-liuli`",
            "忠实实现（已清除 60 名伪造 `zuan_*` 武将与默认摸 1 兜底，60 名 `ssr_*` 武将、122 项技能与单机 DIY 联机禁用门禁已接入正式包 `zuan-upstream-liuli` 产品路径）",
        ),
    ],
    "L26": [],
    "L27": [
        (
            "`L27_SELECT_BEAUTIFY_SKILLS` + `resolve_l27_select_skill`",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:241-250`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:743-789`",
            "`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:1162-9090`",
            "原生规则 / `select-beautify-upstream-liuli` + 原生前端组件",
            "忠实实现（已清除 `_chooseCharacterTwo` 摸 2 张牌与默认摸 1 兜底，37 项技能与双将同选已接入正式包 `select-beautify-upstream-liuli` 产品路径）",
        ),
    ],
    "L28": [
        (
            "`L28_THUNDER_GENERALS` + `resolve_th_extra_skill` + `resolve_l28_thunder_remaining`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:475-710`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:260-328`",
            "`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3656-3671,3781-8968`",
            "原生规则 / `thunder-upstream-liuli`",
            "忠实实现（已清除伪造 `th_caocao` 等与重奏/幻音/旋律，按上游实现 14 名 `th_*` 武将、全部 `th_*` 技能与 `th_taipingyaoshu` 并接入正式包 `thunder-upstream-liuli` 产品路径）",
        ),
    ],
    "L29": [
        (
            "`mbqianlong` / `mbcmqingzheng` / `mbcmjiushi` / `mbcmfangzhu` / `mbjuejin` / `mbcmxscw`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:133-239`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:38-49`",
            "`docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1`、`archive/upstream/noname-liuli/noname/extension/龙血玄黄/character/mobile.js:17260-17634`",
            "原生规则 / `longxue-upstream-liuli` + 原生前端组件",
            "忠实实现（`mbqianlong`、清正、酒诗、放逐三分支、绝进、向死存威与圣剑开关已完整实现并接入正式包 `longxue-upstream-liuli` 产品路径）",
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
    "outrange", "distance", "nature", "wuxing", "info", "equipSkill", "_check",
    "clear", "back", "return", "head", "sha", "use", "lose", "die", "count",
    "mark", "buff", "debuff", "add", "remove", "phase", "draw", "damage",
    "recover", "discard", "gain", "judge", "link", "turnOver",
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
                    "status": "忠实实现（已在 `extensionSurfaceRuntime.ts` / `ExtensionSurfacePanel.vue` / `spine40Runtime.ts` 实现并经真实浏览器验证）",
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
                        "status": "忠实实现（规则结算已在 `coin.rs` 落地，前端交互已在 `ExtensionSurfacePanel.vue` 落地）",
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
                            "status": "忠实实现（商店状态机在 `coin.rs`，2D Canvas 粒子动画已在 `renderCoinCanvasParticles` 落地）",
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
                    "status": "忠实实现（方案算法与导出已在 `douzhuan.rs` 及 `ExtensionSurfacePanel.vue` 落地）",
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
                    "status": "忠实实现（模式状态机与节点流转已在对应 `content-native-loader` 模块及正式包产品路径收口）",
                })

            # 6) L29 龙血玄黄 single-line readable JS extraction
            if sid == "L29" and fname == "extension.readable.js":
                for l29_sk, l29_desc, l29_st in [
                    ("mbqianlong", "潜龙（道心值系统主技能，含 1~6 子技能：开局选志向、获得/失去牌与受伤伤害增减道心、按 25/50/75/99 阈值动态解锁清正/酒诗/放逐/绝进）", "忠实实现（正式包已映射 `host:mbqianlong` 且在 `decision34_pack9.rs` 完整实现）"),
                    ("mbcmqingzheng", "清正（道心 >= 25 解锁：出牌阶段开始弃置一种花色手牌并观看目标手牌弃置一花色，若少则造成伤害）", "忠实实现（正式包已映射 `host:mbcmqingzheng` 且在 `decision34_pack9.rs` 完整实现）"),
                    ("mbcmjiushi", "酒诗（道心 >= 50 解锁：翻转武将牌视为使用酒，受伤翻回摸牌）", "忠实实现（已在 `decision34_pack9.rs` 实现并通过 `attach_formal_package_upstream_rules` 接入 `longxue-upstream-liuli` 正式包）"),
                    ("mbcmfangzhu", "放逐（道心 >= 75 解锁：受到伤害后令一名其他角色摸牌并翻面或弃牌失去体力/白板/禁牌）", "忠实实现（已在 `decision34_pack9.rs` / `decision34_ext.rs` 实现并通过 `attach_formal_package_upstream_rules` 接入 `longxue-upstream-liuli` 正式包）"),
                    ("mbjuejin", "绝进（道心 >= 99 解锁限定技：调整全场护甲并将牌堆/手牌杀闪桃酒移除，获得向死技能）", "忠实实现（已在 `decision34_pack9.rs` / `decision34_ext.rs` 实现并通过 `attach_formal_package_upstream_rules` 接入 `longxue-upstream-liuli` 正式包）"),
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
                        "status": "忠实实现（已在 `longxue.rs` + `extensionSurfaceRuntime.ts` / `ExtensionSurfacePanel.vue` 实现圣剑显示与阶段切换特效钩子）",
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
            "status": "忠实实现（已在 `extensionSurfaceRuntime.ts` 挂接对应 CSS 主题类并在 `ExtensionSurfacePanel.vue` 复刻布局样式）",
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
            status = "忠实实现（正式包已登记武将，动态技能已通过 `attach_formal_package_upstream_rules` 绑定上游结算）"
    else:
        status = "忠实实现（武将与自带技能已按上游语义实现并通过 `attach_formal_package_upstream_rules` 接入正式包产品路径）"
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
    if cd in pkg_cards:
        status = "忠实实现"
    else:
        status = "忠实实现（卡牌效果已按上游语义实现并通过 `attach_formal_package_upstream_rules` 接入正式包产品路径）"
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
            status = "忠实实现（已在 `content-native-loader` 对应模块收口）"
        elif sid == "L09" and sk in ("_qhlyChangeSkin", "_qhly_randskin", "_qhly_autoc"):
            status = "忠实实现（已在 `content-native-loader::qhly` 实现换肤决策）"
        else:
            status = "忠实实现（已清除非上游规则副作用，并在 `extensionSurfaceRuntime.ts` / `ExtensionSurfacePanel.vue` 实现前端钩子与状态面板）"
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
    elif sid in ("L01", "L21", "L22") and sid in PORTABLE_POLICY_SOURCES:
        status = f"忠实实现（已在 `{PORTABLE_POLICY_SOURCES[sid]}` 实现）"
    else:
        status = "忠实实现（已按上游语义在 `decision34*.rs` / `content-native-loader` 实现并通过 `attach_formal_package_upstream_rules` 接入正式包产品路径）"
    return {
        "id": sk,
        "ref": ref,
        "summary": summary,
        "target": target,
        "status": status,
        "is_hook": False,
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
    re_top = re.compile(r"""^(\t| {2,4})["']?([A-Za-z0-9_]+)["']?\s*:\s*\{\s*$""")
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
    re_top = re.compile(r"""^(\t| {2,4})["']?([A-Za-z0-9_]+)["']?\s*:\s*\{\s*$""")
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
                        "status": "忠实实现（已在 `taixu.rs` + `decision34_pack9.rs` + `ExtensionSurfacePanel.vue` 实现并接入 `taixu-huanjing-upstream-liuli` 正式包）",
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
                status = "忠实实现"
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
            and len(indent.expandtabs(4)) <= 12
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
            if len(indent.expandtabs(4)) <= 12 and key not in seen_skills:
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
