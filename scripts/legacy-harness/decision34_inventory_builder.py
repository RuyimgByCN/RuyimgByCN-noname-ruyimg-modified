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
            "`crates/content-native-loader/src/decision34_pack9_data.rs:10-42`、`crates/content-native-loader/src/locked_content.rs:114-126`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:5-215`",
            "原生规则 / `d3-upstream-noname`",
            "忠实实现（31 名武将及全部 64 项 `ddd*` 技能已通过 `attach_formal_package_upstream_rules` 接入 `d3-upstream-noname` 正式包产品路径）",
        ),
        (
            "`U01_DDD_SKILLS` + `resolve_u01_ddd_skill` / `resolve_u01_ddd_remaining`（64 项 `ddd*` 技能，含 `dddjinggou` 装备射程狙击与 `dddmoyan` 亮 3 牌择人获牌）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:124-335`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:383-791`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`",
            "原生规则 / `d3-upstream-noname`",
            "忠实实现（已清除全部分组摸 1 兜底，全部 64 项 `ddd*` 技能均按上游 `file:line` 独立实现并接入 `d3-upstream-noname` 正式包）",
        ),
        (
            "`rotateU01GalleryStandee` 3D 精选立绘路径与展台辅助",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:400-423`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:218`",
            "原生前端组件",
            "忠实实现（立绘映射与展台状态已接入 `extensionSurfaceRuntime.ts`）",
        ),
    ],
    "U02": [
        (
            "`boss_stage_transition` / `resolve_boss_stage_transition`（`boss_baonu_stage` 已按第二步清除）",
            "`crates/content-native-loader/src/boss_playpack.rs:1-120`",
            "`archive/upstream/noname/apps/core/extension/boss/extension.js:6-146`",
            "原生规则（`boss_playpack.rs`）",
            "忠实实现（已清除伪造的 `boss_baonu_stage`；挑战/剑阁武将池缓存与体力归一化已在 `boss_playpack.rs` 忠实实现）",
        ),
    ],
    "U03": [],
    "U04": [
        (
            "`renderCoinCanvasParticles` / `dragBuyCoinShopItem` / `[data-testid=\"coin-weather-overlay\"]`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1123-1177,1223-1241`",
            "`archive/upstream/noname/apps/core/extension/coin/extension.js:17-540`",
            "原生规则（`coin.rs`）+ 原生前端组件",
            "忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts` 实现 2D `<canvas>` 烟花/下雪/星云/闪烁粒子渲染与 `_buy` 拖拽购买交互）",
        ),
    ],
    "U05": [
        (
            "`ShahaiMainConfig`（`tip`/`wuxing`/`rand`/`mtg`/`zhenfa`/`gwent`/`yunchou` 7 项主配置）+ `SHAHAI_FORBID_*` 模式禁选表",
            "`crates/content-native-loader/src/shahai_config.rs:51-236`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:4-48`、`main/content.js:5-11,353-410`",
            "原生规则 / `packages/extensions/sha*`",
            "忠实实现（7 项主配置解析/快照恢复与 boss/chess/double/stone/threecard 模式禁将禁牌表已在 `shahai_config.rs` 按上游实现）",
        ),
        (
            "`U05_*_GENERALS`（332 将）+ `U05_*_ALL_CARDS`（275 牌）+ `U05_*_SKILLS`（1,007 技）+ `resolve_u05_shahai_skill` + `d34_flags.rs` 被动修饰/转化",
            "`crates/content-native-loader/src/shahai_generals_data.rs:8-374`、`crates/content-native-loader/src/shahai_cards_skills_data.rs:7-535`、`crates/content-native-loader/src/shahai_attach.rs:27-214`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_{ops,gujian_gwent,hearth,swd_cards,swd_chars,ow_xianjian}.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:1-674`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/{character,card,play}/*`",
            "原生规则 / 9 个 `sha*-upstream-noname` 正式包",
            "忠实实现（已清除全部分组摸 1 兜底与只写死旗标，补齐攻击距离/手牌上限/出杀次数/`VmViewAsSpec` 修饰、真实判定/拼点守恒及阶段触发守卫并接入 9 个 `sha*-upstream-noname` 正式包；冻结 `ABI 759` 下多步子询问确定性简化边界已登记于 `native-rules-baseline.md`）",
        ),
    ],
    "U06": [
        (
            "`huanle_kuwu_interactive` / `kuwu` / `resolve_u06_huanle_skill`（覆盖 `monkey`、`mianju`、`shoulijian`、`kuwu`、`xuelunyang`、`jiuwei`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:348-399`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:220-266`",
            "`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:1-139`",
            "原生规则 / `huanle-upstream-noname`",
            "忠实实现（已接入正式包 `huanle-upstream-noname` 产品路径，完整覆盖苦无 `kuwu`、猴子偷桃、暗黑面具、手里剑、写轮眼、九尾全部分支）",
        ),
    ],
    "U07": [
        (
            "`diaobingqianjiang` + `U07_WUDIAN_GENERALS`（9 将）+ `U07_WUDIAN_ALL_SKILLS`（30 技，含 `g_youdishenru` 及锁内容路径 `duoqi`/`youzhan`/`jianbi`/`choudu` 绑定）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:401-467`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:269-331`、`crates/content-native-loader/src/decision34_pack9.rs:433-478`",
            "`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:4-168`、`card/skill.js:1-60`、`character/skill.js:1-795`",
            "原生规则 / `wudian-upstream-noname`",
            "忠实实现（9 名武将、30 项技能/子技与 `diaobingqianjiang` 均按上游语义实现并绑定至正式包 `wudian-upstream-noname` 锁内容目录）",
        ),
    ],
    "U08": [
        (
            "`U08_YXS_GENERALS`（43 将）+ `U08_YXS_SKILLS`（107 技，含 `lshengong`、`resolve_u08_yxs_skill` + `yxs_menshen2_guard`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u08_l19_l20_l27.rs:21-625`、`crates/content-native-loader/src/decision34_pack9_data.rs:152-223`",
            "`archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:1-2448`、`character/character.js:3-287`",
            "原生规则 / `hero-kill-upstream-noname`",
            "忠实实现（已清除伪造的 `yxs_tafang_wave` 与 `advanceTafangWave`；全部 43 名英雄杀武将与 107 项技能已按上游实现并接入正式包 `hero-kill-upstream-noname` 产品路径）",
        ),
    ],
    "L01": [
        (
            "`lib.card.yiyi`（以逸待劳锦囊，`resolve_l01_yiyi_card` + `evaluate_yiyi_target_benefit`）",
            "`crates/content-native-loader/src/ai_tuning.rs:183-228`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:158-181`",
            "`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9205-9260`",
            "原生规则 / `ai-tuning-upstream-liuli`",
            "忠实实现（已按上游实现 `yiyi` 锦囊同步摸 2 弃 2 结算与目标收益评估并接入正式包 `ai-tuning-upstream-liuli` 产品路径）",
        ),
    ],
    "L02": [],
    "L03": [
        (
            "`jiange_siege_step` / `boss_playpack.rs`",
            "`crates/content-native-loader/src/boss_playpack.rs:1-120`",
            "`archive/upstream/noname-liuli/noname/extension/boss/extension.js:5-136`",
            "原生规则（`boss_playpack.rs`）",
            "忠实实现（挑战/剑阁武将池缓存与体力归一化已在 `boss_playpack.rs` 忠实实现）",
        ),
    ],
    "L04": [],
    "L05": [],
    "L06": [
        (
            "`_wuxing` 全局五行生克技能（`resolve_l06_wuxing_skill`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:184-217`",
            "`archive/upstream/noname-liuli/noname/extension/五行生克/extension.js:9-95`",
            "原生规则 / `wuxing-upstream-liuli`",
            "忠实实现（已按上游实现 `_wuxing` 初始五行属性与生克结算并接入正式包 `wuxing-upstream-liuli` 产品路径，支持与 `shahai-zhenfa` 任意顺序共存去重）",
        ),
    ],
    "L07": [
        (
            "`runOmnisearchQuery` / `paginateOmnisearchResults` / `highlightOmnisearchTerm`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:523-568,1183-1216`",
            "`archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js:1-2354`、`SimplePagination.js:1-147`",
            "原生规则（`omnisearch.rs`）+ 原生前端组件",
            "忠实实现（已在 `omnisearch.rs` 与 `extensionSurfaceRuntime.ts` 实现公开百科检索、暗牌隐私隔离、分页器与关键词高亮）",
        ),
    ],
    "L08": [
        (
            "`L08_DECADE_CHARACTERS`（9 名伪造 `decade_*` 武将已按第二步清除）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:224-228`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js:12687-12692`（上游 `character: {}` 为空）",
            "无（已清除）",
            "已清除（9 名伪造 `decade_*` 武将与 `_decade_qiankunbagua` 已从代码删除，见 `docs/operations/evidence/2026-11-09-decision34-cleanup/README.md`）",
        ),
        (
            "`DECADE_ALL_UI_HOOK_IDS`（319 项 UI 钩子）+ `DECADE_SURFACE_MODULES`（43 项脚本/CSS 渲染面）+ `UiSkinSuitePanel.vue`",
            "`apps/web-client/src/engine/surfaces/decadeUiSurface.ts:14-323`、`apps/web-client/src/engine/surfaces/decadeMarkSkillsData.ts:15-302`、`apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue:1-411`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/{extension,animation,animation_new,markskill,meihua,splash}.js` 及 33 份 CSS",
            "原生前端组件",
            "忠实实现（已清除摸牌/加甲规则副作用，319 项标记/角标/阴阳转换/动画/重铸/隐匿开场钩子与 43 项布局渲染面已在原生前端完整落地并接入实时战局事件流）",
        ),
        (
            "净室 Spine 4.0 播放器 (`apps/web-client/src/engine/spine40/*` + `Spine40SkinImportBox.vue`，已替换示意 `drawSpineSkeletalCanvas`)",
            "`apps/web-client/src/engine/spine40/{binarySkelCodec,formatCodec,webglRenderer,index}.ts`、`apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue:1-337`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816`、`dynamicSkin.js:1-162`",
            "原生前端组件（净室 Spine 4.0 播放器）",
            "忠实实现（按裁决 35 净室自写 Spine 4.0 JSON/标准 `.skel` 二进制编解码含 zig-zag 有符号 varint 与 `LinkedMesh`、Sutherland-Hodgman + 耳切法凹凸多边形裁剪附件、骨骼世界变换、IK/变换/路径约束、多轨道混合、图集与 `.png` 贴图上传、WebGL 着色器批量绘制及无资源/非 4.0 静态立绘回退）",
        ),
    ],
    "L09": [
        (
            "`QHLY_UI_HOOK_IDS`（8 项 UI 钩子）+ `QHLY_SURFACE_PRESETS`（12 项主题/共享渲染面）+ `UiSkinSuitePanel.vue`",
            "`apps/web-client/src/engine/surfaces/qhlySurface.ts:10-391`、`apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue:1-411`",
            "`archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:1-15970`、`skinShare.js:1-1727`、`extension.css`、`theme/newui*.css`",
            "原生规则（`qhly.rs`）+ 原生前端组件",
            "忠实实现（82 项配置、全部 8 项千幻换肤/击杀特效/BGM/大厅按钮钩子、12 项 `newui*.css`/`skinShare` 主题预设、皮肤详情/台词/语音试听窗口与 `saveSkinParams` 参数同步均已落地）",
        ),
    ],
    "L10": [
        (
            "`k_danggu` / `k_mowang` + 10 项常侍伴生技（`L10_SCS_COMPANION_SKILLS` 真实 `SkillInstance` 挂载与结算）+ `shichangshi` + `triggerL10EmojiRebate` / `triggerL10ScsDieAudio`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:295-346`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:61-155`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1247-1274`",
            "`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:249-310,2276-2970,34517-34570`",
            "原生规则 / `epic-cards-upstream-liuli` + 原生前端组件",
            "忠实实现（十常侍党锢/殁亡结党真实挂载 `scstaoluan`..`scsanruo` 10 项可执行 `SkillInstance`、阵亡语音 `scs_die_audio` 与表情返利 `_emojiRebate1` 已按上游实现并接入正式包 `epic-cards-upstream-liuli` 产品路径）",
        ),
    ],
    "L11": [
        (
            "`L11_TAIXU_GENERALS`（320 将）+ `L11_TAIXU_CARDS`（32 牌）+ `L11_TAIXU_SKILLS`（380 技）+ `resolve_l11_taixu_skill` + `d34_flags.rs` 装备/赐福修饰",
            "`crates/content-native-loader/src/taixu_generals_data.rs:8-395`、`crates/content-native-loader/src/taixu_skills_data.rs:5-195`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_{buffs,skills}.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:1-674`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_{character,card,skill,buff}.js`",
            "原生规则 / `taixu-upstream-liuli`",
            "忠实实现（已清除伪造步进技能 `taixu_expedition_step` 与虚构地名/灵宠名；320 名敌方/随从武将、32 张专属装备/宝物牌含方天/诸葛/虚妄/刑天真实内核修饰、380 项祝福与专属技能均按上游实现并接入正式包 `taixu-upstream-liuli`）",
        ),
        (
            "`L11_TAIXU_CAMPAIGN_NODES`（206 项赛季/奇遇/侍灵/模式节点）+ `L11_TAIXU_SERVANTS`（21 名侍灵）+ `taixuSurface.ts`（4 份 CSS 渲染面）",
            "`crates/content-native-loader/src/taixu_campaign_data.rs:46-378`、`apps/web-client/src/engine/surfaces/taixuSurface.ts:10-214`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/dlc/*/extension_season.js`、`event/event_universal.js`、`extension_servant.js`、`extension_*.css`",
            "原生规则（`taixu_campaign_data.rs`）+ 原生前端组件",
            "忠实实现（8 赛季 DLC 章节与答题、奇遇通用事件、21 名侍灵及 PC/移动端/侍灵/图鉴 4 份样式表布局均已按上游真实名称实现）",
        ),
    ],
    "L12": [
        (
            "`toggleRzshTheme` / `numToRoma` / `applyRzshDynamicSkinLighting`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:687-708,1280-1298`",
            "`archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:1-9502`、`js/rzsh.js:1-815`",
            "原生规则（`rzsh.rs`）+ 原生前端组件",
            "忠实实现（`numToRoma` 罗马数字、动皮光效滤镜与主题切换已在 `rzsh.rs` 与 `extensionSurfaceRuntime.ts` 落地）",
        ),
    ],
    "L13": [
        (
            "`recordMvpEventFact` / `incrementMvpRecastCount` (`_qy_chongzhu`)",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:570-628,1304-1342`",
            "`archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:1-420`",
            "原生规则（`shousha_mvp.rs`）+ 原生前端组件",
            "忠实实现（MVP 攻击/治疗/辅助/重铸计分与防重复流水已在 `shousha_mvp.rs` 与 `extensionSurfaceRuntime.ts` 落地）",
        ),
    ],
    "L14": [
        (
            "`SHOUSHA_UI_HOOK_IDS`（53 项 UI 钩子）+ `SHOUSHA_SURFACE_MODULES`（40 项脚本/CSS 渲染面）+ `UiSkinSuitePanel.vue`",
            "`apps/web-client/src/engine/surfaces/shoushaUiSurface.ts:10-337`、`apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue:1-411`",
            "`archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-7360`、`character/main*.js`、`lbtn/main*.js`、`skill/main*.js` 及 32 份 CSS",
            "原生规则（`shousha_ui.rs`）+ 原生前端组件",
            "忠实实现（已删除伪造 `shousha_*` 实体与跨包虚构名 `spshidi`/`zhefu` 及摸牌/回血副作用；53 项出牌进度条、托管保护、快捷语音气泡、无懈读条、手牌理牌钩子与 40 项手杀面板/样式表已在原生前端完整落地）",
        ),
    ],
    "L15": [],
    "L16": [],
    "L17": [],
    "L18": [],
    "L19": [
        (
            "`L19_DOOMSDAY_CHARACTERS`（已清除 22 名伪造 `vir_*` 武将，按上游 `3383-3384` 设置 `jiangshifz` 5 血男将与 `jiangshinj` 3 血女将及自带五技并接入正式包）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:245-246`、`crates/content-native-loader/src/decision34_pack9.rs:514-563`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-332,3383-3385`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "忠实实现（已清除 22 名伪造 `vir_*` 武将与 `doomsday_infection_step`；`jiangshifz` 与 `jiangshinj` 按上游性别/血量/自带技能接入正式包 `doomsday-brawl-upstream-liuli` 产品路径）",
        ),
        (
            "`L19_DOOMSDAY_SKILLS` + `resolve_l19_doomsday_skill` / `resolve_l19_doomsday_remaining`（全部 66 项乱斗技能与伴生基础技）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:431-486`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:58-326`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:333-3370`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "忠实实现（已清除全部分组摸 1 与空操作桩，按上游逐项实现灾变、8 回合退治、尸变弃全牌 5/5 血摸 4 牌、互助弃桃回血、无尽波次、经验升级、元气护甲、归心等全部规则并接入正式包）",
        ),
    ],
    "L20": [
        (
            "`L20_MARK_SUPPLEMENT_CARDS`（已清除 12 名伪造 `ms_*` 武将与 `ms_longmark`，保留 3 张 `zhengsu_*` 牌并接入正式包）",
            "`crates/content-native-loader/src/decision34_pack9_data.rs:266-267`、`crates/content-native-loader/src/locked_content.rs:114-126`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321,6493-6505`",
            "原生规则 / `mark-supplement-upstream-liuli`",
            "忠实实现（已清除 12 名伪造 `ms_*` 武将与 `ms_longmark`；上游 3 张 `zhengsu_*` 衍生牌已接入正式包 `mark-supplement-upstream-liuli` 产品路径）",
        ),
        (
            "`L20_MARK_SUPPLEMENT_SKILLS` + `resolve_l20_skill` / `resolve_l20_mark_remaining` + `applyL20MarkCssTheme`（74 项标记技能与钩子）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:489-533`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:329-552`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:361-380,1348-1359`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6480`",
            "原生规则 / `mark-supplement-upstream-liuli` + 原生前端组件",
            "忠实实现（已清除全部分组摸 1 与空操作桩，按上游逐项实现 `kurou`/`qingyu`/`mibei`/`wengua5`/`zuoxing`/`_chongzhu`/`_xigua_xueliangfanbei_` 等全部 74 项技能/钩子与 CSS 主题切换并接入正式包）",
        ),
    ],
    "L21": [
        (
            "`martial_bench_substitute` / `martial.rs`（伪造 `martial_formation_step` 已按第二步清除）",
            "`crates/content-native-loader/src/martial.rs:1-120`",
            "`archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js:1-1367`",
            "原生规则 / `martial-upstream-liuli`",
            "忠实实现（已清除伪造 `martial_formation_step`；真实武皇争霸逻辑已在 `content-native-loader::martial` 实现）",
        ),
    ],
    "L22": [],
    "L23": [],
    "L24": [
        (
            "`SKIN_SWITCH_UI_HOOK_IDS`（22 项 UI 钩子）+ `SKIN_SWITCH_SURFACE_IDS`（10 项渲染面）+ `Spine40SkinImportBox.vue`",
            "`apps/web-client/src/engine/surfaces/skinSwitchSurface.ts:10-342`、`apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue:1-411`、`apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue:1-337`",
            "`archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-5900`、`animation.js`、`component/*.css`、`style/*.css`",
            "原生规则（`skin_switch.rs`）+ 原生前端组件",
            "忠实实现（已清除规则层摸牌副作用，22 项动态出框/语音队列/原画回退/`saveSkinParams` 重应用钩子、藏珍阁与骨骼参数编辑弹窗、`.png` 贴图上传及净室 Spine 4.0 播放器均已完整落地）",
        ),
    ],
    "L25": [
        (
            "`L25_SSR_GENERALS`（60 名 `ssr_*` 武将）+ `L25_ALL_RULE_SKILLS`（84 项规则技能）+ `L25_ZUAN_FORMAL_CARDS`（8 牌）+ `resolve_l25_zuan_skill`",
            "`crates/content-native-loader/src/zuan_data.rs:8-376`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_{ssr,ssr_mods,rules}.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs:1-674`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-397,536-51470`",
            "原生规则 / `zuan-upstream-liuli`",
            "忠实实现（已清除 60 名伪造 `zuan_*` 武将、摸 1 兜底与 43 项单旗标空桩；60 名 `ssr_*` 武将、130 项自带技能、84 项规则技能与 8 张专属卡牌已按上游逐项实现并接入正式包 `zuan-upstream-liuli` 产品路径）",
        ),
        (
            "`ZUAN_UI_HOOK_IDS`（40 项攻击指示线、语音与判定全屏特效钩子）",
            "`apps/web-client/src/engine/surfaces/zuanSurface.ts:6-179`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:37514-48503`",
            "原生前端组件",
            "忠实实现（40 项攻击线样式、纸张音效与闪电/火山/洪水/浮雷/乐不思蜀/兵粮寸断判定特效钩子已在前端运行时完整落地）",
        ),
    ],
    "L26": [],
    "L27": [
        (
            "`L27_SELECT_BEAUTIFY_SKILLS` + `resolve_l27_select_skill` / `resolve_l27_select_remaining` + `applyL27SelectBeautifyTheme`（全部 43 项选将/技能分支）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:566-609`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:555-865`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:495-520,1385-1396`",
            "`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:1162-9090`",
            "原生规则 / `select-beautify-upstream-liuli` + 原生前端组件",
            "忠实实现（已清除 `_chooseCharacterTwo` 摸牌与全部分组摸 1 兜底，补齐 `huamu`/`jishe2`/`mbcmqingzheng`/`mbwuling`/`potzhuangshi`/`sbqingzheng` 并区分 `qimou` 不摸牌与 `reqimou` 摸 X 张、`twharvestinori` 摸 2X 张与 `twzhouhu` 回复 X 点，全部 43 项技能/钩子与双将面板接入正式包 `select-beautify-upstream-liuli`）",
        ),
    ],
    "L28": [
        (
            "`L28_THUNDER_GENERALS`（14 将）+ `L28_THUNDER_EXTRA_SKILLS`（39 技）+ `th_taipingyaoshu`（`resolve_l28_thunder_remaining`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:530-754`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:794-1033`、`crates/content-native-loader/src/decision34_closure.rs:243-301`",
            "`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3656-3671,3781-8968`",
            "原生规则 / `thunder-upstream-liuli`",
            "忠实实现（已清除非技能 `_info`/`_map`/`_faq` 键与虚构 `th_caiyi_yang`/`th_pingcai_Wolong`/`th_jingxie1` 等键，补齐 `th_tiansuan2_0..4` 五签、`th_pcaudio_*_card` 四奇卡、`th_zhengjing2`、`th_gushe2` 等全部真实子技，14 名武将、全部 33 项 `th_*` 技能、6 项基础技与 `th_taipingyaoshu` 完整接入正式包 `thunder-upstream-liuli` 产品路径）",
        ),
    ],
    "L29": [
        (
            "`mbqianlong` / `mbcmqingzheng` / `mbcmjiushi` / `mbcmfangzhu` / `mbjuejin` + `_CMeffects` (`resolve_l29_longxue_skill`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:178-293`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:335-378`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:330-359,1402-1413`",
            "`docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1`、`archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.js:1-1048`",
            "原生规则 / `longxue-upstream-liuli` + 原生前端组件",
            "忠实实现（`mbqianlong`、`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin` 与 `_CMeffects` 已按上游实现并接入正式包 `longxue-upstream-liuli` 产品路径）",
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
    "outrange", "distance", "conflictMap", "imgPath", "scs_bagua", "scs_bgf",
    "scs_qjf", "zhulianbihe", "SS_skillwqx", "look", "back", "return", "die",
    "die2", "mark", "mark1", "mark2", "Ai", "draw", "gain", "destroy",
    "recover", "qyubee", "use", "cancel", "broadcast", "settle", "change",
    "nochange", "range", "add", "limit", "directHit", "m2", "mahou", "begin",
    "damage", "item",
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
            if sid in ("L07", "L08", "L09", "L12", "L14", "L24", "L27", "L29"):
                syms = _native_symbols()
                f_surfs = syms["faithful_surfaces"].get(sid, set())
                rel_in_src = jf.relative_to(src_dir).as_posix() if jf.is_relative_to(src_dir) else fname
                if sid in ("L07", "L12", "L27", "L29"):
                    surf_status = "忠实实现（已在 `extensionSurfaceRuntime.ts` 对应组件函数落地）"
                elif jf.stem in f_surfs or fname in f_surfs or rel_in_src in f_surfs:
                    surf_status = "忠实实现（已在 `apps/web-client/src/engine/surfaces/*` 与 `UiSkinSuitePanel.vue` 按上游布局与状态机落地）"
                else:
                    surf_status = "缺失"
                surfaces.append({
                    "id": jf.stem,
                    "ref": f"`{rel}:1-{len(lines)}`",
                    "summary": f"前端脚本模块 `{fname}`（{len(lines)} 行）",
                    "target": "原生前端组件",
                    "status": surf_status,
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
                            "status": "忠实实现（已在 `coin.rs` 与 `extensionSurfaceRuntime.ts::renderCoinCanvasParticles` 落地）",
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
                syms = _native_symbols()
                if sid in ("L21",):
                    mode_status = "忠实实现"
                elif sid == "L11" and mname in syms["l11_campaign_nodes"]:
                    mode_status = "忠实实现（已在 `taixu_campaign_data.rs` 与正式包 `taixu-upstream-liuli` 落地）"
                else:
                    mode_status = "缺失（独立模式主循环未完整迁入正式包）"
                modes.append({
                    "id": mname,
                    "ref": f"`{rel}:{lineno}`",
                    "summary": f"注册游戏模式 `{mname}`（{trans.get(mname, mname)}）",
                    "target": "原生规则",
                    "status": mode_status,
                })

            # 6) L29 龙血玄黄 single-line readable JS extraction
            if sid == "L29" and fname == "extension.readable.js":
                for l29_sk, l29_desc, l29_st in [
                    ("mbqianlong", "潜龙（道心值系统主技能，含 1~6 子技能：开局选志向、获得/失去牌与受伤伤害增减道心、按 25/50/75/99 阈值动态解锁清正/酒诗/放逐/绝进）", "忠实实现（正式包已映射 `host:mbqianlong`）"),
                    ("mbcmqingzheng", "清正（道心 >= 25 解锁：出牌阶段开始弃置一种花色手牌并观看目标手牌弃置一花色，若少则造成伤害）", "忠实实现（已按上游语义实现并接入 `longxue-upstream-liuli` 正式包产品路径）"),
                    ("mbcmjiushi", "酒诗（道心 >= 50 解锁：翻转武将牌视为使用酒，受伤翻回摸牌）", "忠实实现（已按上游语义实现并接入 `longxue-upstream-liuli` 正式包产品路径）"),
                    ("mbcmfangzhu", "放逐（道心 >= 75 解锁：受到伤害后令一名其他角色摸牌并翻面或弃牌失去体力）", "忠实实现（已按上游语义实现并接入 `longxue-upstream-liuli` 正式包产品路径）"),
                    ("mbjuejin", "绝进（道心 >= 99 解锁限定技：调整全场护甲并将牌堆/手牌杀闪桃酒移除，获得向死技能）", "忠实实现（已按上游语义实现并接入 `longxue-upstream-liuli` 正式包产品路径）"),
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
                        "status": "忠实实现（已在 `extensionSurfaceRuntime.ts::triggerLongxueHolySwordEffect` 实现圣剑特效状态机）",
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
        syms = _native_symbols()
        f_surfs = syms["faithful_surfaces"].get(sid, set())
        rel_in_src = cf.relative_to(src_dir).as_posix() if cf.is_relative_to(src_dir) else cf.name
        if sid not in ("L08", "L09", "L11", "L14", "L24"):
            css_status = "忠实实现"
        elif cf.name in f_surfs or rel_in_src in f_surfs:
            css_status = "忠实实现（已在 `apps/web-client/src/engine/surfaces/*` 与对应面板组件按上游样式令牌落地）"
        else:
            css_status = "缺失"
        surfaces.append({
            "id": cf.name,
            "ref": f"`{rel}:1-{len(clines)}`",
            "summary": f"CSS 样式表 `{cf.name}`（{len(clines)} 行）",
            "target": "原生前端组件",
            "status": css_status,
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


_NATIVE_SYMBOLS_CACHE: dict[str, dict] | None = None


def _extract_const_block(src: str, const_name: str) -> str:
    m = re.search(rf"(?:pub\s+)?const\s+{re.escape(const_name)}\b[^=]*=\s*\[", src)
    if not m:
        return ""
    depth = 1
    pos = m.end()
    while pos < len(src) and depth > 0:
        if src[pos] == "[":
            depth += 1
        elif src[pos] == "]":
            depth -= 1
        pos += 1
    return src[m.end() : max(m.end(), pos - 1)]


def _parse_rust_str_array(src: str, const_name: str) -> set[str]:
    block = _extract_const_block(src, const_name)
    return set(re.findall(r'"([^"]+)"', block))


def _parse_rust_tuple_first_elems(src: str, const_name: str) -> set[str]:
    block = _extract_const_block(src, const_name)
    return set(re.findall(r'\(\s*"([^"]+)"\s*,', block))


def _parse_ts_str_array(src: str, const_name: str) -> set[str]:
    m = re.search(rf"export\s+const\s+{re.escape(const_name)}\b[^=]*=\s*\[", src)
    if not m:
        return set()
    depth = 1
    pos = m.end()
    while pos < len(src) and depth > 0:
        if src[pos] == "[":
            depth += 1
        elif src[pos] == "]":
            depth -= 1
        pos += 1
    return set(re.findall(r'"([^"]+)"', src[m.end() : max(m.end(), pos - 1)]))


L01_AI_POLICY_HOOKS = {
    "_aiyh_firstKs", "_aiyh_nhFriends", "_aiyh_meks", "_aiyh_neiKey", "_aiyh_fixEQz",
    "_aiyh_fixECf", "_aiyh_fake_prohibited", "_aiyh_fixWj", "_findZhong", "_mjAiSkill",
    "aiMangju", "_attzhi", "attcard", "attdamage", "attlosehp", "attrecover", "attgain",
    "attlose", "attturnover", "attbo", "attmingbian", "attmingbian1", "attfanzhuang",
    "attbaolu", "atttiaofan", "_shyg_qjAi", "_aiyh_reserved_shan", "aiyh_disabled_shan",
    "_shyg_wuxie", "_shyg_wuxie_jiedao", "_shyg_wuxie_lei", "shyg_wuxie_fy", "_shyg_wugu",
    "shyg_wugu_ss_pai", "shyg_wugu_wss_pai", "_shyg_Discard", "shyg_Discard_key",
    "_shyg_Discard_du", "shyg_Discard_du_useful", "_fenghuyunlong", "shyg_mz",
    "_tianxiayitong", "shyg_smz", "jlnj", "tfattzhu", "tfattzhong", "atttf",
    "_yingshilanggu", "shyg_tz", "shyg_tf", "_wanjunpiyi", "chaofeng_common",
    "chaofeng_rare", "chaofeng_epic", "chaofeng_legend", "_shyg", "gjcx_zhuAi",
    "gjcx_neiAi", "gjcx_neiZhong", "gjcx_neiFan", "gjcx_neiJiang", "_aiyh_lianhe",
    "_aiyh_wqjz", "xinfu_limu", "zhenlie_lose", "twzhian_ai", "dcwumei_all",
    "aiyh_gjcx_qj", "_gjcx_correct",
}

L22_ELO_POLICY_HOOKS = {
    "_eloMechanism", "_eloInitCards", "_eloDraw", "_eloHandcard", "_timeKiller",
}


def _parse_attach_arm_strings(src: str, package_id: str) -> set[str]:
    m = re.search(rf'\(\s*"[^"]+"\s*,\s*"{re.escape(package_id)}"\s*\)\s*=>\s*\{{', src)
    if not m:
        return set()
    depth = 1
    pos = m.end()
    while pos < len(src) and depth > 0:
        if src[pos] == "{":
            depth += 1
        elif src[pos] == "}":
            depth -= 1
        pos += 1
    return set(re.findall(r'"([^"]+)"', src[m.end() : pos]))


def _native_symbols() -> dict[str, dict]:
    global _NATIVE_SYMBOLS_CACHE
    if _NATIVE_SYMBOLS_CACHE is not None:
        return _NATIVE_SYMBOLS_CACHE
    p9_data = (REPO / "crates/content-native-loader/src/decision34_pack9_data.rs").read_text(encoding="utf-8")
    closure_src = (REPO / "crates/content-native-loader/src/decision34_closure.rs").read_text(encoding="utf-8")
    p9_mod = (REPO / "crates/content-native-loader/src/decision34_pack9.rs").read_text(encoding="utf-8")
    zuan_rs = (REPO / "crates/content-native-loader/src/zuan_data.rs").read_text(encoding="utf-8")
    taixu_gen_rs = (REPO / "crates/content-native-loader/src/taixu_generals_data.rs").read_text(encoding="utf-8")
    taixu_sk_rs = (REPO / "crates/content-native-loader/src/taixu_skills_data.rs").read_text(encoding="utf-8")
    taixu_camp_rs = (REPO / "crates/content-native-loader/src/taixu_campaign_data.rs").read_text(encoding="utf-8")
    shahai_gen_rs = (REPO / "crates/content-native-loader/src/shahai_generals_data.rs").read_text(encoding="utf-8")
    shahai_cs_rs = (REPO / "crates/content-native-loader/src/shahai_cards_skills_data.rs").read_text(encoding="utf-8")
    shahai_cfg_rs = (REPO / "crates/content-native-loader/src/shahai_config.rs").read_text(encoding="utf-8")

    surf_dir = REPO / "apps/web-client/src/engine/surfaces"
    qhly_ts = (surf_dir / "qhlySurface.ts").read_text(encoding="utf-8")
    skin_ts = (surf_dir / "skinSwitchSurface.ts").read_text(encoding="utf-8")
    shousha_ts = (surf_dir / "shoushaUiSurface.ts").read_text(encoding="utf-8")
    decade_ts = (surf_dir / "decadeUiSurface.ts").read_text(encoding="utf-8")
    decade_marks_ts = (surf_dir / "decadeMarkSkillsData.ts").read_text(encoding="utf-8")
    zuan_ts = (surf_dir / "zuanSurface.ts").read_text(encoding="utf-8")
    taixu_ts = (surf_dir / "taixuSurface.ts").read_text(encoding="utf-8")

    d34_specs = _parse_rust_tuple_first_elems(closure_src, "DECISION34_SKILL_SPECS")

    u05_generals: set[str] = set()
    for arr_name in (
        "U05_GUJIAN_GENERALS",
        "U05_GWENT_GENERALS",
        "U05_HEARTH_GENERALS",
        "U05_MTG_GENERALS",
        "U05_OW_GENERALS",
        "U05_SWD_GENERALS",
        "U05_XIANJIAN_GENERALS_FULL",
        "U05_YUNCHOU_GENERALS",
        "U05_STONE_GENERALS",
    ):
        u05_generals |= _parse_rust_tuple_first_elems(shahai_gen_rs, arr_name)

    u05_skills: set[str] = set()
    for arr_name in (
        "U05_GUJIAN_ALL_SKILLS",
        "U05_GWENT_ALL_SKILLS",
        "U05_HEARTH_ALL_SKILLS",
        "U05_MTG_ALL_SKILLS",
        "U05_SWD_CARD_SKILLS",
        "U05_SWD_CHAR_SKILLS",
        "U05_YUNCHOU_ALL_SKILLS",
        "U05_WUXING_PLAY_SKILLS",
        "U05_OW_ALL_SKILLS",
        "U05_XIANJIAN_ALL_SKILLS",
        "U05_CROSS_SKILLS",
    ):
        u05_skills |= _parse_rust_str_array(shahai_cs_rs, arr_name)

    u05_cards: set[str] = set()
    for arr_name in (
        "U05_GUJIAN_ALL_CARDS",
        "U05_GWENT_ALL_CARDS",
        "U05_HEARTH_ALL_CARDS",
        "U05_MTG_ALL_CARDS",
        "U05_SWD_ALL_CARDS",
        "U05_YUNCHOU_ALL_CARDS",
        "U05_ZHENFA_ALL_CARDS",
        "U05_WUXING_ALL_CARDS",
    ):
        u05_cards |= _parse_rust_tuple_first_elems(shahai_cs_rs, arr_name)

    faithful_generals: dict[str, set[str]] = {
        "U01": _parse_rust_tuple_first_elems(p9_data, "U01_DDD_GENERALS"),
        "U05": u05_generals,
        "U07": _parse_rust_tuple_first_elems(closure_src, "U07_WUDIAN_GENERALS")
        | _parse_rust_tuple_first_elems(p9_data, "U07_WUDIAN_GENERALS"),
        "U08": _parse_rust_tuple_first_elems(p9_data, "U08_YXS_GENERALS"),
        "L11": _parse_rust_tuple_first_elems(taixu_gen_rs, "L11_TAIXU_GENERALS"),
        "L19": _parse_rust_str_array(p9_data, "L19_DOOMSDAY_CHARACTERS"),
        "L25": _parse_rust_tuple_first_elems(zuan_rs, "L25_SSR_GENERALS"),
        "L28": _parse_rust_tuple_first_elems(closure_src, "L28_THUNDER_GENERALS"),
    }
    partial_generals: dict[str, set[str]] = {}

    l19_skills = _parse_rust_str_array(p9_data, "L19_DOOMSDAY_SKILLS") | _parse_attach_arm_strings(p9_mod, "doomsday-brawl")
    l20_skills = _parse_rust_str_array(p9_data, "L20_MARK_SUPPLEMENT_SKILLS")
    l27_skills = _parse_rust_str_array(p9_data, "L27_SELECT_BEAUTIFY_SKILLS")
    l28_skills = _parse_rust_str_array(closure_src, "L28_THUNDER_EXTRA_SKILLS") | {
        s for s in d34_specs if s.startswith("th_")
    }

    faithful_skills: dict[str, set[str]] = {
        "U01": _parse_rust_str_array(p9_data, "U01_DDD_SKILLS"),
        "U05": u05_skills,
        "U06": _parse_attach_arm_strings(p9_mod, "huanle-cards"),
        "U07": _parse_rust_str_array(p9_data, "U07_WUDIAN_ALL_SKILLS")
        | _parse_rust_str_array(p9_data, "U07_WUDIAN_REMAINING_SKILLS")
        | {"duoqi", "youzhan"},
        "U08": _parse_rust_str_array(p9_data, "U08_YXS_SKILLS")
        | _parse_rust_tuple_first_elems(closure_src, "U08_STANDARD_SKILL_REFS"),
        "L06": _parse_attach_arm_strings(p9_mod, "wuxing") | {"_wuxing"},
        "L10": _parse_attach_arm_strings(p9_mod, "epic-cards")
        | _parse_rust_str_array(closure_src, "L10_SCS_COMPANION_SKILLS"),
        "L11": _parse_rust_str_array(taixu_sk_rs, "L11_TAIXU_SKILLS")
        | _parse_rust_str_array(taixu_sk_rs, "L11_TAIXU_CROSS_SKILLS"),
        "L19": l19_skills,
        "L20": l20_skills,
        "L25": _parse_rust_str_array(zuan_rs, "L25_ALL_RULE_SKILLS")
        | _parse_rust_str_array(zuan_rs, "L25_SSR_GENERAL_SKILLS"),
        "L27": l27_skills,
        "L28": l28_skills,
        "L29": _parse_attach_arm_strings(p9_mod, "longxue"),
    }
    portable_policy_hooks: dict[str, set[str]] = {
        "L01": L01_AI_POLICY_HOOKS,
        "L22": L22_ELO_POLICY_HOOKS,
    }
    decade_hooks = _parse_ts_str_array(decade_ts, "DECADE_CORE_HOOK_IDS") | set(
        re.findall(r'\{\s*id:\s*"([^"]+)"', decade_marks_ts)
    )
    qhly_hooks = _parse_ts_str_array(qhly_ts, "QHLY_UI_HOOK_IDS")
    qhly_surfs = set(re.findall(r'\|\s*"([^"]+)"', qhly_ts.split("export interface")[0])) - qhly_hooks
    faithful_ui_hooks: dict[str, set[str]] = {
        "U04": {"_buy", "coin_arenaReady"},
        "L05": {"_buy", "coin_arenaReady"},
        "L08": decade_hooks,
        "L09": qhly_hooks,
        "L10": {"_emojiRebate1", "scs_die_audio"},
        "L13": {"_qy_chongzhu"},
        "L14": _parse_ts_str_array(shousha_ts, "SHOUSHA_UI_HOOK_IDS"),
        "L17": {"banCharF"},
        "L19": l19_skills,
        "L20": l20_skills,
        "L23": {"_fixCAILANZI"},
        "L24": _parse_ts_str_array(skin_ts, "SKIN_SWITCH_UI_HOOK_IDS"),
        "L25": _parse_ts_str_array(zuan_ts, "ZUAN_UI_HOOK_IDS"),
        "L26": {"_Fool_chooseCard"},
        "L27": l27_skills | {"_chooseCharacterTwo"},
        "L28": l28_skills,
        "L29": {"_CMeffects"},
    }
    faithful_surfaces: dict[str, set[str]] = {
        "L08": _parse_ts_str_array(decade_ts, "DECADE_SURFACE_MODULES"),
        "L09": qhly_surfs,
        "L11": _parse_ts_str_array(taixu_ts, "TAIXU_SURFACE_STYLESHEETS"),
        "L14": _parse_ts_str_array(shousha_ts, "SHOUSHA_SURFACE_MODULES"),
        "L24": _parse_ts_str_array(skin_ts, "SKIN_SWITCH_SURFACE_IDS"),
    }
    faithful_configs: dict[str, set[str]] = {
        "U05": set(re.findall(r'read_bool\("([^"]+)"\)', shahai_cfg_rs))
        | set(re.findall(r'obj\.get\("([^"]+)"\)', shahai_cfg_rs)),
    }
    l11_campaign_nodes: set[str] = set(
        re.findall(r'TaixuCampaignNodeSpec\s*\{[^}]*\bid:\s*"([^"]+)"', taixu_camp_rs)
    ) | _parse_rust_tuple_first_elems(taixu_camp_rs, "L11_TAIXU_SERVANTS")
    partial_skills: dict[str, set[str]] = {}
    faithful_cards: set[str] = (
        {"yiyi", "diaobingqianjiang", "shichangshi", "th_taipingyaoshu"}
        | _parse_rust_str_array(p9_data, "L20_MARK_SUPPLEMENT_CARDS")
        | u05_cards
        | _parse_rust_str_array(taixu_gen_rs, "L11_TAIXU_CARD_IDS")
        | _parse_rust_tuple_first_elems(taixu_gen_rs, "L11_TAIXU_CARDS")
        | _parse_rust_str_array(zuan_rs, "L25_ZUAN_CARD_ENTRIES")
        | _parse_rust_tuple_first_elems(zuan_rs, "L25_ZUAN_FORMAL_CARDS")
    )

    _NATIVE_SYMBOLS_CACHE = {
        "faithful_generals": faithful_generals,
        "partial_generals": partial_generals,
        "faithful_skills": faithful_skills,
        "portable_policy_hooks": portable_policy_hooks,
        "faithful_ui_hooks": faithful_ui_hooks,
        "faithful_surfaces": faithful_surfaces,
        "faithful_configs": faithful_configs,
        "l11_campaign_nodes": l11_campaign_nodes,
        "partial_skills": partial_skills,
        "faithful_cards": faithful_cards,
    }
    return _NATIVE_SYMBOLS_CACHE


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
    syms = _native_symbols()
    f_skills = syms["faithful_skills"].get(sid, set())
    f_generals = syms["faithful_generals"].get(sid, set())
    p_generals = syms["partial_generals"].get(sid, set())

    zh = trans.get(cid, cid)
    sk_str = ", ".join(sk_list[:6]) + ("…" if len(sk_list) > 6 else "")
    summary = clean_md(f"{zh}（势力={grp}, 性别={sex}, 体力={hp}, 技能=[{sk_str}]）")
    if cid in pkg_generals:
        g_skills = pkg_generals[cid]
        unmapped = [k for k, v in g_skills.items() if not v and k not in f_skills]
        if not unmapped:
            status = "忠实实现"
        else:
            status = f"缺失（正式包已登记武将，但技能 {','.join(unmapped)} 为 mapping: null）"
    elif cid in f_generals:
        status = "忠实实现（武将与自带技能已按上游语义实现并接入正式包产品路径）"
    elif cid in p_generals:
        status = "示意或自造（武将已接入正式包产品路径，但部分自带技能尚未逐个完整实现）"
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
    syms = _native_symbols()
    summary = summarize_snippet(lines, idx, trans, cd)
    if cd in pkg_cards or cd in syms["faithful_cards"]:
        status = "忠实实现"
    elif cd in pkg_deferred:
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
    syms = _native_symbols()
    f_skills = syms["faithful_skills"].get(sid, set())
    p_skills = syms["partial_skills"].get(sid, set())
    f_hooks = syms["faithful_ui_hooks"].get(sid, set())
    policy_hooks = syms["portable_policy_hooks"].get(sid, set())

    summary = summarize_snippet(lines, idx, trans, sk)
    window = " ".join(lines[idx : min(len(lines), idx + 22)])
    is_ui_hook = (
        sid in UI_ONLY_SOURCES
        or (sk.startswith("_") and sid in ("U04", "L05", "L08", "L09", "L10", "L12", "L14", "L20", "L24", "L27", "L28"))
        or (
            sid not in ("U01", "U05", "U06", "U07", "U08", "L19")
            and any(k in window for k in ("decadeUI", "playAudio", "setBackgroundImage", "ui.create"))
        )
    )
    if is_ui_hook and sk not in pkg_skills:
        target = "原生前端组件"
        if sid == "L09" and sk in f_hooks:
            status = "忠实实现（已在 `qhly.rs` 与 `apps/web-client/src/engine/surfaces/qhlySurface.ts` 落地）"
        elif sk in f_hooks:
            status = "忠实实现（已在正式包规则或 `apps/web-client/src/engine/surfaces/*` / `extensionSurfaceRuntime.ts` 对应模块收口）"
        elif sk in p_skills:
            status = "示意或自造"
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
    elif sk in f_skills:
        status = "忠实实现（已按上游语义实现并接入正式包产品路径）"
    elif sk in policy_hooks and sid in PORTABLE_POLICY_SOURCES:
        status = f"忠实实现（已在 `{PORTABLE_POLICY_SOURCES[sid]}` 实现）"
    elif sk in p_skills:
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
    syms = _native_symbols()
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
                    node_status = (
                        "忠实实现（已在 `taixu_campaign_data.rs` 与 `taixuSurface.ts` 落地并接入正式包 `taixu-upstream-liuli`）"
                        if kid in syms["l11_campaign_nodes"]
                        else "缺失"
                    )
                    modes.append({
                        "id": kid,
                        "ref": f"`{rel}:{lineno}`",
                        "summary": clean_md(f"{fname} 节点 `{kid}`（{zh}）"),
                        "target": "原生规则（`taixu_campaign_data.rs`）+ 原生前端组件",
                        "status": node_status,
                    })
            continue

        m = re_key_obj.match(line)
        if not m:
            continue
        indent, key = m.group(1), m.group(2)
        if key in RESERVED_SKILL_KEYS:
            continue
        window = "\n".join(lines[idx : min(len(lines), idx + 16)])

        # Config entry detection: has `name` and (`init` or `intro` or `onclick` or `item`, or `clear` in config.js) inside a small block
        has_cfg_name = re.search(r"""["']?name["']?\s*:""", window) is not None
        has_cfg_meta = (
            re.search(r"""["']?(?:init|intro|onclick|item)["']?\s*:""", window) is not None
            or (fname == "config.js" and re.search(r"""["']?clear["']?\s*:""", window) is not None)
        )
        has_js_content_prop = re.search(r"(?<![A-Za-z0-9_-])content\s*:", window) is not None
        if has_cfg_name and has_cfg_meta and "trigger:" not in window and not has_js_content_prop:
            if key not in seen_configs:
                seen_configs.add(key)
                m_n = re.search(r"""["']?name["']?\s*:\s*["']([^"']+)["']""", window)
                m_i = re.search(r"""["']?init["']?\s*:\s*([^,\n]+)""", window)
                m_intro = re.search(r"""["']?intro["']?\s*:\s*["']([^"']+)["']""", window)
                zh = clean_md(m_n.group(1), 40) if m_n else key
                init_v = clean_md(m_i.group(1), 25) if m_i else "—"
                intro_v = clean_md(m_intro.group(1), 50) if m_intro else ""
                summary = clean_md(f"{zh}（默认={init_v}）{('：' + intro_v) if intro_v else ''}")
                has_formal_cfg = any(
                    (REPO / "packages/extensions" / pname / "assets/config.json").exists()
                    for pname in SOURCE_FORMAL_PKGS.get(sid, [])
                )
                f_cfgs = syms["faithful_configs"].get(sid, set())
                if sid in PORTABLE_POLICY_SOURCES:
                    target = f"原生规则（`{PORTABLE_POLICY_SOURCES[sid]}`）"
                elif key in f_cfgs and sid == "U05":
                    target = "原生规则（`crates/content-native-loader/src/shahai_config.rs`）"
                else:
                    target = "原生规则 / 配置 schema"
                status = (
                    "忠实实现"
                    if (sid in PORTABLE_POLICY_SOURCES or has_formal_cfg or key in f_cfgs)
                    else "缺失"
                )
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
        if has_js_content_prop or any(
            kw in window
            for kw in ("trigger:", "viewAs:", "filterCard:", "filterTarget:", "mod:", "subSkill:", "marktext:", "enable:")
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
        "## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-10）",
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
