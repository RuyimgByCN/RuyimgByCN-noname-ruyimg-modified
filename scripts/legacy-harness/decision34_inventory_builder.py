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
            "简化近似（待复核：31 名武将及全部 64 项 `ddd*` 技能已通过 `attach_formal_package_upstream_rules` 接入 `d3-upstream-noname` 正式包产品路径）",
        ),
        (
            "`U01_DDD_SKILLS` + `resolve_u01_ddd_skill` / `resolve_u01_ddd_remaining`（64 项 `ddd*` 技能，含 `dddjinggou` 装备射程狙击与 `dddmoyan` 亮 3 牌择人获牌）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:124-335`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:354-774`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:4-6368`",
            "原生规则 / `d3-upstream-noname`",
            "简化近似（待复核：已清除全部分组摸 1 兜底，全部 64 项 `ddd*` 技能均按上游 `file:line` 独立实现并接入 `d3-upstream-noname` 正式包）",
        ),
        (
            "`rotateU01GalleryStandee` 3D 精选立绘路径与展台辅助",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:400-423`",
            "`archive/upstream/noname/apps/core/extension/3D精选/character/character.js:218`",
            "原生前端组件",
            "示意（待对照：立绘映射与展台状态已接入 `extensionSurfaceRuntime.ts`）",
        ),
    ],
    "U02": [
        (
            "`boss_stage_transition` / `resolve_boss_stage_transition`（`boss_baonu_stage` 已按第二步清除）",
            "`crates/content-native-loader/src/boss_playpack.rs:1-120`",
            "`archive/upstream/noname/apps/core/extension/boss/extension.js:6-146`",
            "原生规则（`boss_playpack.rs`）",
            "简化近似（待复核：已清除伪造的 `boss_baonu_stage`；挑战/剑阁武将池缓存与体力归一化已在 `boss_playpack.rs` 落地）",
        ),
    ],
    "U03": [],
    "U04": [
        (
            "`renderCoinCanvasParticles` / `dragBuyCoinShopItem` / `[data-testid=\"coin-weather-overlay\"]`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1123-1177,1223-1241`",
            "`archive/upstream/noname/apps/core/extension/coin/extension.js:17-540`",
            "原生规则（`coin.rs`）+ 原生前端组件",
            "示意（待对照：已在 `coin.rs` 与 `extensionSurfaceRuntime.ts` 实现 2D `<canvas>` 烟花/下雪/星云/闪烁粒子渲染与 `_buy` 拖拽购买交互）",
        ),
    ],
    "U05": [
        (
            "`ShahaiMainConfig`（`tip`/`wuxing`/`rand`/`mtg`/`zhenfa`/`gwent`/`yunchou` 7 项主配置）+ `SHAHAI_FORBID_*` 模式禁选表",
            "`crates/content-native-loader/src/shahai_config.rs:51-236`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:4-48`、`main/content.js:5-11,353-410`",
            "原生规则 / `packages/extensions/sha*`",
            "简化近似（待复核：7 项主配置解析/快照恢复与 boss/chess/double/stone/threecard 模式禁将禁牌表已在 `shahai_config.rs` 按上游实现）",
        ),
        (
            "`U05_*_GENERALS`（332 将）+ `U05_*_ALL_CARDS`（275 牌）+ `IMPLEMENTED_U05_VM_SKILLS`（919 项已验证 VM 技能）+ `host_skill_bindings.rs` + `intrinsic_named_ext.rs` + `d34_u05_mods.rs`",
            "`crates/content-native-loader/src/shahai_attach.rs:1-450`、`crates/content-native-loader/src/shahai_implemented_skills.rs:1-190`、`crates/engine-native-content/src/content/host/intrinsic_named_ext.rs:1-531`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_u05_mods.rs:1-533`",
            "`archive/upstream/noname/apps/core/extension/杀海拾遗/{character,card,play}/*`",
            "原生规则 / 9 个 `sha*-upstream-noname` 正式包",
            "简化近似（待复核：275 张专属卡牌经真实 `PlayCard` 与 `intrinsic_named_ext.rs` 结算、装备牌绑定真实 `equip_skills`、25 项原生 `HostBehavior` 零遮蔽、`d34_u05_mods.rs` 显式修饰与全部 919 项 VM 技能真实效果已全量接通；已彻底清除 201 项同名卡牌伪技能与全部死旗标空桩，1,621 项已接入原生落点）",
        ),
    ],
    "U06": [
        (
            "`monkey` / `mianju` / `shoulijian` / `kuwu` / `xuelunyang` / `jiuwei`（6 张卡牌 + 5 项同名装备技能，零 VM 旁路）",
            "`crates/engine-native-kernel/src/kernel/monkey.rs:8-128`、`crates/engine-native/src/engine/play_card.rs:696-773`、`crates/engine-native-content/src/content/host/step/skills_status.rs:665-723`、`crates/engine-native-content/src/content/host/step/skills_damage.rs:570-637`、`crates/engine-native-content/src/content/host/step/skills_targets.rs:212-301`、`crates/engine-native-content/src/content/host/step/skills_recovery.rs:287-295`、`crates/content-native-loader/src/host_skill_bindings.rs:118-122`、`packages/extensions/huanle-upstream-noname/assets/cards.json:1-72`",
            "`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:1-127`、`card/skill.js:1-146`、`card/list.js:1-11`",
            "原生规则 / `huanle-upstream-noname`",
            "已复核等价（11/11 项经 `u06-verifier` 独立逐行对照上游 JS 与 6 维真实对局测试复核等价，见 `docs/operations/evidence/2026-11-12-u06-huanle-review/README.md`）",
        ),
    ],
    "U07": [
        (
            "`diaobingqianjiang` + `U07_WUDIAN_GENERALS`（9 将）+ `U07_WUDIAN_ALL_SKILLS`（30 技，含 `g_youdishenru` 及锁内容路径 `duoqi`/`youzhan`/`jianbi`/`choudu` 绑定）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:345-465`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:208-301`、`crates/content-native-loader/src/decision34_pack9.rs:433-478`",
            "`archive/upstream/noname/apps/core/extension/玩点论杀/card/card.js:4-168`、`card/skill.js:1-60`、`character/skill.js:1-795`",
            "原生规则 / `wudian-upstream-noname`",
            "简化近似（待复核：9 名武将、30 项技能/子技与 `diaobingqianjiang` 均按上游语义实现并绑定至正式包 `wudian-upstream-noname` 锁内容目录）",
        ),
    ],
    "U08": [
        (
            "`U08_YXS_GENERALS`（43 将）+ `U08_YXS_SKILLS`（107 技，含 `lshengong`、`resolve_u08_yxs_skill` + `yxs_menshen2_guard`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u08_l19_l20_l27.rs:21-625`、`crates/content-native-loader/src/decision34_pack9_data.rs:152-223`",
            "`archive/upstream/noname/apps/core/extension/英雄杀/character/skill.js:1-2448`、`character/character.js:3-287`",
            "原生规则 / `hero-kill-upstream-noname`",
            "简化近似（待复核：已清除伪造的 `yxs_tafang_wave` 与 `advanceTafangWave`；全部 43 名英雄杀武将与 107 项技能已按上游实现并接入正式包 `hero-kill-upstream-noname` 产品路径）",
        ),
    ],
    "L01": [
        (
            "`lib.card.yiyi`（以逸待劳锦囊，`resolve_l01_yiyi_card` + `evaluate_yiyi_target_benefit`）",
            "`crates/content-native-loader/src/ai_tuning.rs:183-228`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:142-165`",
            "`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9205-9260`",
            "原生规则 / `ai-tuning-upstream-liuli`",
            "简化近似（待复核：已按上游实现 `yiyi` 锦囊同步摸 2 弃 2 结算与目标收益评估并接入正式包 `ai-tuning-upstream-liuli` 产品路径）",
        ),
    ],
    "L02": [],
    "L03": [
        (
            "`jiange_siege_step` / `boss_playpack.rs`",
            "`crates/content-native-loader/src/boss_playpack.rs:1-120`",
            "`archive/upstream/noname-liuli/noname/extension/boss/extension.js:5-136`",
            "原生规则（`boss_playpack.rs`）",
            "简化近似（待复核：挑战/剑阁武将池缓存与体力归一化已在 `boss_playpack.rs` 落地）",
        ),
    ],
    "L04": [],
    "L05": [],
    "L06": [
        (
            "`_wuxing` 全局五行生克技能（`resolve_l06_wuxing_skill`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:172-204`",
            "`archive/upstream/noname-liuli/noname/extension/五行生克/extension.js:9-95`",
            "原生规则 / `wuxing-upstream-liuli`",
            "简化近似（待复核：已按上游实现 `_wuxing` 初始五行属性与生克结算并接入正式包 `wuxing-upstream-liuli` 产品路径，支持与 `shahai-zhenfa` 任意顺序共存去重）",
        ),
    ],
    "L07": [
        (
            "`runOmnisearchQuery` / `paginateOmnisearchResults` / `highlightOmnisearchTerm`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:523-568,1183-1216`",
            "`archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js:1-2354`、`SimplePagination.js:1-147`",
            "原生规则（`omnisearch.rs`）+ 原生前端组件",
            "示意（待对照：已在 `omnisearch.rs` 与 `extensionSurfaceRuntime.ts` 实现公开百科检索、暗牌隐私隔离、分页器与关键词高亮）",
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
            "示意（待对照：已清除摸牌/加甲规则副作用，319 项标记/角标/阴阳转换/动画/重铸/隐匿开场钩子与 43 项布局渲染面已在原生前端完整落地并接入实时战局事件流）",
        ),
        (
            "净室 Spine 4.0 播放器 (`apps/web-client/src/engine/spine40/*` + `Spine40SkinImportBox.vue`，已替换示意 `drawSpineSkeletalCanvas`)",
            "`apps/web-client/src/engine/spine40/{binarySkelCodec,formatCodec,webglRenderer,index}.ts`、`apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue:1-337`",
            "`archive/upstream/noname-liuli/noname/extension/十周年UI/animation.js:1-1816`、`dynamicSkin.js:1-162`",
            "原生前端组件（净室 Spine 4.0 播放器）",
            "示意（待对照：按裁决 35 净室自写 Spine 4.0 JSON/标准 `.skel` 二进制编解码含 zig-zag 有符号 varint 与 `LinkedMesh`、Sutherland-Hodgman + 耳切法凹凸多边形裁剪附件、骨骼世界变换、IK/变换/路径约束、多轨道混合、图集与 `.png` 贴图上传、WebGL 着色器批量绘制及无资源/非 4.0 静态立绘回退）",
        ),
    ],
    "L09": [
        (
            "`QHLY_UI_HOOK_IDS`（8 项 UI 钩子）+ `QHLY_SURFACE_PRESETS`（12 项主题/共享渲染面）+ `UiSkinSuitePanel.vue`",
            "`apps/web-client/src/engine/surfaces/qhlySurface.ts:10-391`、`apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue:1-411`",
            "`archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js:1-15970`、`skinShare.js:1-1727`、`extension.css`、`theme/newui*.css`",
            "原生规则（`qhly.rs`）+ 原生前端组件",
            "示意（待对照：82 项配置、全部 8 项千幻换肤/击杀特效/BGM/大厅按钮钩子、12 项 `newui*.css`/`skinShare` 主题预设、皮肤详情/台词/语音试听窗口与 `saveSkinParams` 参数同步均已落地）",
        ),
    ],
    "L10": [
        (
            "`k_danggu` / `k_mowang` + 10 项常侍伴生技（`L10_SCS_COMPANION_SKILLS` 真实 `SkillInstance` 挂载与结算）+ `shichangshi` + `triggerL10EmojiRebate` / `triggerL10ScsDieAudio`",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:295-346`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:61-155`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:1247-1274`",
            "`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:249-310,2276-2970,34517-34570`",
            "原生规则 / `epic-cards-upstream-liuli` + 原生前端组件",
            "简化近似（待复核：十常侍党锢/殁亡结党真实挂载 `scstaoluan`..`scsanruo` 10 项可执行 `SkillInstance`、阵亡语音 `scs_die_audio` 与表情返利 `_emojiRebate1` 已按上游实现并接入正式包 `epic-cards-upstream-liuli` 产品路径）",
        ),
    ],
    "L11": [
        (
            "`L11_TAIXU_GENERALS`（320 将）+ `L11_TAIXU_CARDS`（32 牌）+ `L11_TAIXU_SKILLS`（374 项本包技能/赐福）+ `L11_TAIXU_OWN_NPC_SKILLS`（29 项本包 Boss 技）+ `L11_TAIXU_IMPLEMENTED_CROSS_SKILLS`（164 项跨包 NPC 技）+ 237 项 `package.npack.json` 预编译 `rule-vm` 跨包技 + `d34_l11_mods.rs` + `L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS`（9 项上游未定义悬空引用保持惰性）",
            "`crates/content-native-loader/src/taixu_skills_data.rs:5-311`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_{buffs,skills,npc_a,npc_b}.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/{d34_l11_mods,d34_view_as}.rs`、`packages/extensions/taixu-upstream-liuli/package.npack.json`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_{character,card,skill,buff}.js`、`archive/upstream/noname-liuli/noname/character/*.js`、`archive/upstream/noname/apps/core/character/**/*.js`",
            "原生规则 / `taixu-upstream-liuli`",
            "简化近似（待复核：已清除 6 个 `subSkill` 正则伪影及全部分组桶；374 项本包技、29 项本包 Boss 技、164 项跨包 NPC 技、237 项预编译 `rule-vm` 跨包技、32 张卡牌及 320 名武将全部按上游 `file:line` 落地；`extension_character.js:321-337` 何进诛宦 11 名 NPC 上引用的 9 个上游未定义悬空技能 ID `L11_TAIXU_UPSTREAM_UNDEFINED_SKILL_REFS` 按上游运行时惰性处理，严禁凭空自造）",
        ),
        (
            "`L11_TAIXU_CAMPAIGN_NODES`（206 项节点）+ `TaixuCampaignRun` 赛季分支状态机 + `TaixuCampaignModal.vue`",
            "`crates/content-native-loader/src/taixu_campaign_data.rs:46-485`、`apps/web-client/src/components/battle/modals/TaixuCampaignModal.vue:1-250`",
            "`archive/upstream/noname-liuli/noname/extension/太虚幻境/dlc/*/extension_season.js`、`event/event_universal.js`、`extension_servant.js`、`extension_*.css`",
            "原生规则（`taixu_campaign_data.rs`）+ 原生前端组件",
            "简化近似（待复核：商铺与奇遇统一发放真实上游 `buff_txhj_*` ID，`TaixuCampaignRun` 支持赛季分支图推进与快照往返，`BattleView.vue` 已挂载 `TaixuCampaignModal.vue`）",
        ),
    ],
    "L12": [
        (
            "`toggleRzshTheme` / `numToRoma` / `applyRzshDynamicSkinLighting`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:687-708,1280-1298`",
            "`archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:1-9502`、`js/rzsh.js:1-815`",
            "原生规则（`rzsh.rs`）+ 原生前端组件",
            "示意（待对照：`numToRoma` 罗马数字、动皮光效滤镜与主题切换已在 `rzsh.rs`、`BattleShoushaHud.vue` 与 `extensionSurfaceRuntime.ts` 落地）",
        ),
    ],
    "L13": [
        (
            "`recordMvpEventFact` / `incrementMvpRecastCount` (`_qy_chongzhu`) + `MvpSettlementModal.vue`",
            "`apps/web-client/src/engine/extensionSurfaceRuntime.ts:570-628,1304-1342`、`apps/web-client/src/components/battle/modals/MvpSettlementModal.vue:1-155`",
            "`archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:1-420`",
            "原生规则（`shousha_mvp.rs`）+ 原生前端组件",
            "示意（待对照：MVP 攻击/治疗/辅助/重铸计分与全场结算献花弹窗 `MvpSettlementModal.vue` 已挂载至 `BattleView.vue`）",
        ),
    ],
    "L14": [
        (
            "`SHOUSHA_UI_HOOK_IDS`（53 项 UI 钩子）+ `SHOUSHA_SURFACE_MODULES`（40 项渲染面）+ `BattleShoushaHud.vue` + `HandStrip.vue`「理牌」",
            "`apps/web-client/src/engine/surfaces/shoushaUiSurface.ts:10-337`、`apps/web-client/src/components/battle/BattleShoushaHud.vue:1-190`、`apps/web-client/src/components/battle/HandStrip.vue:1-194`",
            "`archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js:85-7360`、`character/main*.js`、`lbtn/main*.js`、`skill/main*.js` 及 32 份 CSS",
            "原生规则（`shousha_ui.rs`）+ 原生前端组件",
            "示意（待对照：牌桌顶栏阶段旗幡、读秒进度条、将灯徽章与手牌区一键「理牌」已直接挂载于 `BattleBoard.vue` / `HandStrip.vue`）",
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
            "简化近似（待复核：已清除 22 名伪造 `vir_*` 武将与 `doomsday_infection_step`；`jiangshifz` 与 `jiangshinj` 按上游性别/血量/自带技能接入正式包 `doomsday-brawl-upstream-liuli` 产品路径）",
        ),
        (
            "`L19_DOOMSDAY_SKILLS` + `resolve_l19_doomsday_skill` / `resolve_l19_doomsday_remaining`（全部 66 项乱斗技能与伴生基础技）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:431-486`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u08_l19_l20_l27.rs:58-326`",
            "`archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:333-3370`",
            "原生规则 / `doomsday-brawl-upstream-liuli`",
            "简化近似（待复核：已清除全部分组摸 1 与空操作桩，按上游逐项实现灾变、8 回合退治、尸变弃全牌 5/5 血摸 4 牌、互助弃桃回血、无尽波次、经验升级、元气护甲、归心等全部规则并接入正式包）",
        ),
    ],
    "L20": [
        (
            "`L20_MARK_SUPPLEMENT_CARDS` + `MarkSupplementConfig`（3 张 `zhengsu_*` 展示牌 + 15 项扩展配置）",
            "`crates/content-native-loader/src/mark_supplement.rs:1-357`、`crates/content-native-loader/src/decision34_pack9_data.rs:266-267`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6319-6321,6398-6491`",
            "原生规则 / `mark-supplement-upstream-liuli`",
            "已复核等价（复核人: l20-signoff, 日期: 2026-11-12, 3 张整肃展示牌与 15 项扩展配置已通过独立复核与 6 维真实事件对局测试）",
        ),
        (
            "`HostBehavior::L20MarkBatchA` + `HostBehavior::L20MarkBatchB`（28 项技能 + 46 项钩子）+ `applyL20MarkCssTheme`（3 项 CSS 渲染面）",
            "`crates/engine-native-content/src/content/host/step/l20_batch_{a,a_asks,b,b_asks,b_ext,b_mouyi}.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_{l11_mods,view_as,flags}.rs`",
            "`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:577-6308`",
            "原生规则 / `mark-supplement-upstream-liuli` + 原生前端组件",
            "已复核等价 92/95 项（复核人: l20-signoff, 日期: 2026-11-12；全部 28 项技能与 46 项 `ui_hooks` 均按上游 `file:line` 对齐并通过 6 维真实事件对局测试；仅 3 项纯 CSS 表面文件保留 `简化近似`，待总排队第 5 项 UI 阶段对齐）",
        ),
    ],
    "L21": [
        (
            "`martial_bench_substitute` / `martial.rs`（伪造 `martial_formation_step` 已按第二步清除）",
            "`crates/content-native-loader/src/martial.rs:1-120`",
            "`archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js:1-1367`",
            "原生规则 / `martial-upstream-liuli`",
            "简化近似（待复核：已清除伪造 `martial_formation_step`；真实武皇争霸逻辑已在 `content-native-loader::martial` 落地）",
        ),
    ],
    "L22": [],
    "L23": [],
    "L24": [
        (
            "`SKIN_SWITCH_UI_HOOK_IDS`（22 项 UI 钩子）+ `SKIN_SWITCH_SURFACE_IDS`（10 项渲染面）+ `SkinSwitchModal.vue` + `SeatCard.vue` 出框叠加",
            "`apps/web-client/src/engine/surfaces/skinSwitchSurface.ts:10-342`、`apps/web-client/src/components/battle/modals/SkinSwitchModal.vue:1-170`、`apps/web-client/src/components/battle/SeatCard.vue:1-335`",
            "`archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js:176-5900`、`animation.js`、`component/*.css`、`style/*.css`",
            "原生规则（`skin_switch.rs`）+ 原生前端组件",
            "示意（待对照：22 项出框/语音队列/原画回退钩子、座位 `flipX` 出框徽章、藏珍阁与坐标编辑全屏弹窗 `SkinSwitchModal.vue` 及净室 Spine 4.0 播放器均已挂载）",
        ),
    ],
    "L25": [
        (
            "`L25_SSR_GENERALS`（60 名 `ssr_*` 武将）+ `L25_ALL_RULE_SKILLS` + `L25_ZUAN_SKILLS`（124 项规则/特效技）+ `L25_ZUAN_FORMAL_CARDS`（6 牌）",
            "`crates/content-native-loader/src/zuan_data.rs:8-370`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_{ssr,ssr_mods,rules}.rs`、`crates/engine-native-content/src/content/host/intrinsic_named_ext.rs:1-531`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:338-397,536-51470`",
            "原生规则 / `zuan-upstream-liuli`",
            "简化近似（待复核：已清除 5 个 `subSkill` 正则伪影 `wuxie`/`equip`/`res`/`defend`/`zhou` 及 `l25_unhandled_skill:*` 兜底；60 名 `ssr_*` 武将、130 项自带技能、全部规则/音效技能与 6 张专属卡牌均按上游逐项实现并接入正式包 `zuan-upstream-liuli`）",
        ),
        (
            "`ZUAN_UI_HOOK_IDS`（40 项攻击指示线、语音与判定全屏特效钩子）",
            "`apps/web-client/src/engine/surfaces/zuanSurface.ts:6-179`",
            "`archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:37514-48503`",
            "原生前端组件",
            "示意（待对照：40 项攻击线样式、纸张音效与闪电/火山/洪水/浮雷/乐不思蜀/兵粮寸断判定特效钩子已在前端运行时完整落地）",
        ),
    ],
    "L26": [],
    "L27": [
        (
            "`L27_SELECT_BEAUTIFY_SKILLS` + `resolve_l27_select_skill` / `resolve_l27_select_remaining` + `applyL27SelectBeautifyTheme`（全部 43 项选将/技能分支）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs:566-586`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs:555-853`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:495-520,1385-1396`",
            "`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:1162-9090`",
            "原生规则 / `select-beautify-upstream-liuli` + 原生前端组件",
            "简化近似（待复核：已清除 `_chooseCharacterTwo` 摸牌与全部分组摸 1 兜底，补齐 `huamu`/`jishe2`/`mbcmqingzheng`/`mbwuling`/`potzhuangshi`/`sbqingzheng` 并区分 `qimou` 不摸牌与 `reqimou` 摸 X 张、`twharvestinori` 摸 2X 张与 `twzhouhu` 回复 X 点，全部 43 项技能/钩子与双将面板接入正式包 `select-beautify-upstream-liuli`）",
        ),
    ],
    "L28": [
        (
            "`L28_THUNDER_GENERALS`（14 将）+ `L28_THUNDER_EXTRA_SKILLS`（39 技）+ `th_taipingyaoshu`（`resolve_l28_thunder_remaining`）",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:530-747`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:778-1007`、`crates/content-native-loader/src/decision34_closure.rs:243-301`",
            "`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3656-3671,3781-8968`",
            "原生规则 / `thunder-upstream-liuli`",
            "简化近似（待复核：已清除非技能 `_info`/`_map`/`_faq` 键与虚构 `th_caiyi_yang`/`th_pingcai_Wolong`/`th_jingxie1` 等键，补齐 `th_tiansuan2_0..4` 五签、`th_pcaudio_*_card` 四奇卡、`th_zhengjing2`、`th_gushe2` 等全部真实子技，14 名武将、全部 33 项 `th_*` 技能、6 项基础技与 `th_taipingyaoshu` 完整接入正式包 `thunder-upstream-liuli` 产品路径）",
        ),
    ],
    "L29": [
        (
            "`mbqianlong` / `mbcmqingzheng` / `mbcmjiushi` / `mbcmfangzhu` / `mbjuejin` + `_CMeffects` (`resolve_l29_longxue_skill`)",
            "`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs:178-286`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs:305-348`、`apps/web-client/src/engine/extensionSurfaceRuntime.ts:330-359,1402-1413`",
            "`docs/operations/evidence/2026-10-08-e01-longxue-lx1/extension.readable.js:1`、`archive/upstream/noname-liuli/noname/extension/龙血玄黄/专属技能区/手杀ui/skill/main1.js:1-1048`",
            "原生规则 / `longxue-upstream-liuli` + 原生前端组件",
            "简化近似（待复核：`mbqianlong`、`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin` 与 `_CMeffects` 已按上游实现并接入正式包 `longxue-upstream-liuli` 产品路径）",
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
                surfaces.append({
                    "id": jf.stem,
                    "ref": f"`{rel}:1-{len(lines)}`",
                    "summary": f"前端脚本模块 `{fname}`（{len(lines)} 行）",
                    "target": "原生前端组件",
                    "status": "示意（待对照）",
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
                            "status": "简化近似（待复核）",
                        })
                if line.strip().startswith("init()"):
                    modes.append({
                        "id": "boss_play_init",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "加载 `boss_storage_playpackconfig` 挑战/剑阁武将池、归一化体力与 AI 禁选",
                        "target": "原生规则（`boss_playpack.rs`）",
                        "status": "简化近似（待复核）",
                    })
                elif line.strip().startswith("arenaReady()"):
                    modes.append({
                        "id": "boss_play_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "异步加载 `boss`/`versus` 模式武将池与技能并排除 `versus_ladder`",
                        "target": "原生规则（`boss_playpack.rs`）",
                        "status": "简化近似（待复核）",
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
                        "status": "简化近似（待复核）",
                    })
                elif line.strip().startswith("arenaReady()"):
                    modes.append({
                        "id": "cardpile_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "按 `data` 160 张基数与比例配置向 `lib.card.list` 追加牌堆条目（上限 1000）",
                        "target": "原生规则（`pile_bridge.rs`）",
                        "status": "简化近似（待复核）",
                    })

            # 2c) U04 / L05 coin shop & hooks
            if sid in ("U04", "L05"):
                if line.strip().startswith("init()"):
                    modes.append({
                        "id": "coin_init",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "非战棋统率模式下初始化 `_status.coin = 0`",
                        "target": "原生规则（`coin.rs`）",
                        "status": "简化近似（待复核）",
                    })
                elif line.strip().startswith("arenaReady()"):
                    ui_hooks.append({
                        "id": "coin_arenaReady",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "创建顶栏富甲天下金币按钮、商店弹窗与本局获胜 20 金下注入口",
                        "target": "原生规则（`coin.rs`）+ 原生前端组件",
                        "status": "示意（待对照）",
                    })
                elif line.strip().startswith("changeCoin("):
                    modes.append({
                        "id": "game.changeCoin",
                        "ref": f"`{rel}:{lineno}`",
                        "summary": "增减金币、持久化 `lib.config.coin`、触发提示与音效",
                        "target": "原生规则（`coin.rs`）",
                        "status": "简化近似（待复核）",
                    })
                for shop_key, shop_zh in [("firework", "烟花"), ("snow", "下雪"), ("star", "星云"), ("blink", "闪烁")]:
                    if re.match(rf"^\s*{shop_key}\s*:\s*\{{", line) and f"shop_{shop_key}" not in seen_configs:
                        seen_configs.add(f"shop_{shop_key}")
                        surfaces.append({
                            "id": f"haveFun.{shop_key}",
                            "ref": f"`{rel}:{lineno}`",
                            "summary": f"富甲天下商店特效 `{shop_key}`（{shop_zh} Canvas 粒子与启停控制）",
                            "target": "原生规则（`coin.rs`）+ 原生前端组件",
                            "status": "示意（待对照）",
                        })

            # 2d) L15 手杀全扩 / L17 斗转星移 policy hooks
            if sid == "L15" and "var changePlayer=" in line:
                modes.append({
                    "id": "changePlayer_pool",
                    "ref": f"`{rel}:{lineno}`",
                    "summary": "手杀全扩白名单将池表（风林火山/山阴雷/谋攻篇等）与将包映射过滤",
                    "target": "原生规则（`shousha.rs`）",
                    "status": "简化近似（待复核）",
                })
            if sid == "L17" and "function banCharF()" in line:
                ui_hooks.append({
                    "id": "banCharF",
                    "ref": f"`{rel}:{lineno}`",
                    "summary": "斗转星移禁将面板（3 套方案切换、全禁/全开、导入导出、星级评分）",
                    "target": "原生规则（`douzhuan.rs`）+ 原生前端组件",
                    "status": "示意（待对照）",
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
                    "status": "简化近似（待复核）",
                })

            # 6) L29 龙血玄黄 single-line readable JS extraction
            if sid == "L29" and fname == "extension.readable.js":
                for l29_sk, l29_desc in [
                    ("mbqianlong", "潜龙（道心值系统主技能，含 1~6 子技能：开局选志向、获得/失去牌与受伤伤害增减道心、按 25/50/75/99 阈值动态解锁清正/酒诗/放逐/绝进）"),
                    ("mbcmqingzheng", "清正（道心 >= 25 解锁：出牌阶段开始弃置一种花色手牌并观看目标手牌弃置一花色，若少则造成伤害）"),
                    ("mbcmjiushi", "酒诗（道心 >= 50 解锁：翻转武将牌视为使用酒，受伤翻回摸牌）"),
                    ("mbcmfangzhu", "放逐（道心 >= 75 解锁：受到伤害后令一名其他角色摸牌并翻面或弃牌失去体力）"),
                    ("mbjuejin", "绝进（道心 >= 99 解锁限定技：调整全场护甲并将牌堆/手牌杀闪桃酒移除，获得向死技能）"),
                ]:
                    if l29_sk not in seen_skills:
                        seen_skills.add(l29_sk)
                        skills.append({
                            "id": l29_sk,
                            "ref": f"`{rel}:1`",
                            "summary": l29_desc,
                            "target": "原生规则 / `longxue-upstream-liuli`",
                            "status": "简化近似（待复核）",
                        })
                if "_CMeffects" not in seen_skills:
                    seen_skills.add("_CMeffects")
                    ui_hooks.append({
                        "id": "_CMeffects",
                        "ref": f"`{rel}:1`",
                        "summary": "圣剑显示与阶段切换特效钩子（`lib.skill._CMeffects`）",
                        "target": "原生前端组件",
                        "status": "示意（待对照）",
                    })
                if "shengjian" not in seen_configs:
                    seen_configs.add("shengjian")
                    configs.append({
                        "id": "shengjian",
                        "ref": f"`{rel}:1`",
                        "summary": "圣剑显示开关（`config.shengjian.init = true`）",
                        "target": "原生规则（`longxue.rs`）",
                        "status": "简化近似（待复核）",
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
            "status": "示意（待对照）",
        })

    inv = {
        "generals": generals,
        "skills": skills,
        "cards": cards,
        "modes": modes,
        "configs": configs,
        "ui_hooks": ui_hooks,
        "surfaces": surfaces,
    }
    ledger = load_inventory_status_ledger()
    for cat in ("generals", "skills", "cards", "modes", "configs", "ui_hooks", "surfaces"):
        for item in inv[cat]:
            bind_item_status(sid, cat, item, ledger)
    return inv


STATUS_LEDGER_PATH = REPO / "docs" / "migration" / "g0" / "inventory_status.json"
_INVENTORY_STATUS_CACHE: dict[str, dict] | None = None


def load_inventory_status_ledger() -> dict[str, dict]:
    global _INVENTORY_STATUS_CACHE
    if _INVENTORY_STATUS_CACHE is None:
        if STATUS_LEDGER_PATH.exists():
            data = json.loads(STATUS_LEDGER_PATH.read_text(encoding="utf-8"))
            _INVENTORY_STATUS_CACHE = data.get("items", {})
        else:
            _INVENTORY_STATUS_CACHE = {}
    return _INVENTORY_STATUS_CACHE


def format_item_status(entry: dict) -> str:
    st = entry.get("status", "缺失")
    if st == "已复核等价":
        rev = entry.get("reviewer") or "未记录"
        dt = entry.get("review_date") or "未记录"
        up_ref = entry.get("upstream_file_line") or "未记录"
        test = entry.get("test_name") or "未记录"
        return f"已复核等价（复核人: {rev}, 日期: {dt}, 对照: {up_ref}, 测试: {test}）"
    elif st == "简化近似":
        diff = entry.get("difference") or ""
        note = entry.get("note")
        suffix = f" · 注: {note}" if note else ""
        if diff in ("简化近似（待复核）", "示意（待对照）"):
            return f"{diff}{suffix}"
        return f"简化近似（差异: {diff}）{suffix}" if diff else f"简化近似{suffix}"
    elif st == "缺失":
        reason = entry.get("reason")
        return f"缺失（{reason}）" if reason else "缺失"
    elif st == "上游未定义":
        reason = entry.get("reason")
        return f"上游未定义（{reason}）" if reason else "上游未定义"
    return st


def bind_item_status(sid: str, cat: str, item: dict, ledger: dict[str, dict]) -> dict:
    clean_ref = item["ref"].strip("`")
    pk = f"{sid}:{cat}:{item['id']}@{clean_ref}"
    entry = ledger.get(pk)
    if entry:
        item["status"] = format_item_status(entry)
    else:
        diff = "简化近似（待复核）" if cat in ("generals", "skills", "cards", "modes", "configs") else "示意（待对照）"
        item["status"] = diff
    return item


def _classify_general(
    sid: str,
    cid: str,
    ref: str,
    sex: str,
    grp: str,
    hp: str,
    sk_list: list[str],
    trans: dict[str, str],
    pkg_generals: dict[str, dict] | None = None,
) -> dict:
    zh = trans.get(cid, cid)
    sk_str = ", ".join(sk_list[:6]) + ("…" if len(sk_list) > 6 else "")
    summary = clean_md(f"{zh}（势力={grp}, 性别={sex}, 体力={hp}, 技能=[{sk_str}]）")
    return {
        "id": cid,
        "ref": ref,
        "summary": summary,
        "target": "原生规则 / `rule-vm` 包",
        "status": "简化近似（待复核）",
    }


def _classify_card(
    sid: str,
    cd: str,
    ref: str,
    lines: list[str],
    idx: int,
    trans: dict[str, str],
    pkg_cards: dict[str, str] | None = None,
    pkg_deferred: dict[str, str] | None = None,
) -> dict:
    summary = summarize_snippet(lines, idx, trans, cd)
    return {
        "id": cd,
        "ref": ref,
        "summary": summary,
        "target": "原生规则 / `rule-vm` 包",
        "status": "简化近似（待复核）",
    }


def _classify_skill_or_hook(
    sid: str,
    sk: str,
    ref: str,
    lines: list[str],
    idx: int,
    trans: dict[str, str],
    pkg_skills: dict[str, str | None] | None = None,
) -> dict:
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
    if is_ui_hook and (pkg_skills is None or sk not in pkg_skills):
        target = "原生前端组件"
        return {
            "id": sk,
            "ref": ref,
            "summary": summary,
            "target": target,
            "status": "示意（待对照）",
            "is_hook": True,
        }

    target = "原生规则 / `rule-vm` 包"
    return {
        "id": sk,
        "ref": ref,
        "summary": summary,
        "target": target,
        "status": "简化近似（待复核）",
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


def _strip_js_line_for_braces(line: str, in_block_comment: bool) -> tuple[str, bool]:
    out: list[str] = []
    i = 0
    n = len(line)
    in_str: str | None = None
    while i < n:
        ch = line[i]
        nxt = line[i + 1] if i + 1 < n else ""
        if in_block_comment:
            if ch == "*" and nxt == "/":
                in_block_comment = False
                i += 2
            else:
                i += 1
            continue
        if in_str is not None:
            if ch == "\\":
                i += 2
                continue
            if ch == in_str:
                in_str = None
            i += 1
            continue
        if ch == "/" and nxt == "/":
            break
        if ch == "/" and nxt == "*":
            in_block_comment = True
            i += 2
            continue
        if ch in ("'", '"', "`"):
            in_str = ch
            i += 1
            continue
        out.append(ch)
        i += 1
    return "".join(out), in_block_comment


def _compute_subskill_line_mask(lines: list[str]) -> list[bool]:
    mask = [False] * len(lines)
    in_block_comment = False
    subskill_depth = 0
    re_subskill = re.compile(r"(?<![A-Za-z0-9_$])subSkill\s*:\s*\{")
    for idx, raw in enumerate(lines):
        cleaned, in_block_comment = _strip_js_line_for_braces(raw, in_block_comment)
        if subskill_depth > 0:
            mask[idx] = True
            for ch in cleaned:
                if ch == "{":
                    subskill_depth += 1
                elif ch == "}":
                    subskill_depth -= 1
                    if subskill_depth == 0:
                        break
        else:
            m = re_subskill.search(cleaned)
            if m:
                mask[idx] = True
                subskill_depth = 1
                for ch in cleaned[m.end() :]:
                    if ch == "{":
                        subskill_depth += 1
                    elif ch == "}":
                        subskill_depth -= 1
                        if subskill_depth == 0:
                            break
    return mask


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
    subskill_mask = _compute_subskill_line_mask(lines)
    re_top = re.compile(r"""^(\t+| {2,16})["']?([A-Za-z0-9_]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        if subskill_mask[idx]:
            continue
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
    subskill_mask = _compute_subskill_line_mask(lines)
    re_top = re.compile(r"""^(\t| {2,4})["']?([A-Za-z0-9_$]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        if subskill_mask[idx]:
            continue
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
    subskill_mask = _compute_subskill_line_mask(lines)
    re_key_obj = re.compile(r"""^(\s*)["']?([A-Za-z0-9_$\u4e00-\u9fa5]+)["']?\s*:\s*\{\s*$""")
    for idx, line in enumerate(lines):
        if subskill_mask[idx]:
            continue
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
                        "target": "原生规则（`taixu_campaign_data.rs`）+ 原生前端组件",
                        "status": "简化近似（待复核）",
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
                if sid in PORTABLE_POLICY_SOURCES:
                    target = f"原生规则（`{PORTABLE_POLICY_SOURCES[sid]}`）"
                elif sid == "U05":
                    target = "原生规则（`crates/content-native-loader/src/shahai_config.rs`）"
                else:
                    target = "原生规则 / 配置 schema"
                configs.append({
                    "id": key,
                    "ref": f"`{rel}:{lineno}`",
                    "summary": summary,
                    "target": target,
                    "status": "简化近似（待复核）",
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


def count_statuses(inv: dict[str, list[dict]], d34_rows: list[tuple] | None = None) -> dict[str, int]:
    counts = {"已复核等价": 0, "简化近似": 0, "缺失": 0, "上游未定义": 0}
    for cat in ("generals", "skills", "cards", "modes", "configs", "ui_hooks", "surfaces"):
        for item in inv.get(cat, []):
            st = item.get("status", "")
            if st.startswith("已复核等价"):
                counts["已复核等价"] += 1
            elif st.startswith("简化近似") or st.startswith("示意"):
                counts["简化近似"] += 1
            elif st.startswith("上游未定义"):
                counts["上游未定义"] += 1
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
        "## 7. 裁决 34 逐项上游清单与现有实现归类（2026-11-11）",
        "",
        f"- **统计汇总**：上游实体合计 **{total}** 项（武将 {len(inv['generals'])} / 技能 {len(inv['skills'])} / 卡牌 {len(inv['cards'])} / 模式与战役节点 {len(inv['modes'])} / 配置项 {len(inv['configs'])} / 界面钩子 {len(inv['ui_hooks'])} / 渲染面 {len(inv['surfaces'])}）",
        f"- **状态分布**：`已复核等价` **{counts['已复核等价']}** 项 · `简化近似` **{counts['简化近似']}** 项 · `缺失` **{counts['缺失']}** 项 · `上游未定义` **{counts['上游未定义']}** 项",
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
