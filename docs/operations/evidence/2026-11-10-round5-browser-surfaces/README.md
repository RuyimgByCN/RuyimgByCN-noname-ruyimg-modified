# 2026-11-10 第五轮复审纠偏、死旗标清退、`U05 杀海拾遗` 全量忠实闭合、牌桌 HUD/全屏弹窗挂载与如实台账核验

- **核验日期**：2026-11-10
- **代码基线**：`native-convergence` `0ed74fdcd` / `main` `5168ecbe9` / 初版 `886fafafa`
- **冻结协议与版本**：`RULESET_VERSION = 333`、`ARCHIVE_FORMAT_VERSION = 107`（NGSA 107）、`content-schema = 5`、`RULE_VM_ABI_VERSION = 759`、`NETWORK_PROTOCOL_VERSION = 8`（WS8）；14 场标准 golden 场景哈希零变化。

## 1. 本轮解决的五类实质问题与逐项删改清单

### 1.1 `U05 杀海拾遗`：剥离同名卡牌伪技能与死旗标空桩，补齐全部 919 项 VM 技能、25 项原生 `HostBehavior` 与被动修饰（`1,621 / 0 / 0` 全量忠实闭合）

1. **删除笼统旗标前缀兜底、消除伪技能遮蔽并补齐全部真实技能效果**：
   - 在 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs) 移除对 `u05_buff:`、`u05_debuff:`、`u05_mod:` 的笼统前缀加杀次数兜底，改为在 [`d34_u05_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_u05_mods.rs) 按具体技能名与全部 24 项剩余修饰/减益旗标（如 `u05_debuff:bingdong`、`u05_debuff:luan`、`u05_debuff:chengxin`、`u05_mod:shanguang2`、`u05_mod:bfengshi_range`、`u05_mod:qixia_tails` 等）逐条计算攻击距离、被指定距离、手牌上限、出杀上限、防具失效与免伤。
   - 在 [`shahai_attach.rs`](../../../../crates/content-native-loader/src/shahai_attach.rs) 停止将 **201 张专属卡牌名**伪造注册为 `VmSkillDecl`，改为为装备牌正确绑定 `equip_skills`；停止对已有原生 `HostBehavior` 的技能重复注入 `VmSkillDecl` 造成遮蔽，扩充 [`host_skill_bindings.rs`](../../../../crates/content-native-loader/src/host_skill_bindings.rs) 绑定全部 **25 项**原生 `HostBehavior` 技能（含 `diyguhuo`/`diyjiaoxia`/`diyhonglian`/`diykuanggu`/`liyan`/`zonghuo`/`shaoying`/`xiyan`/`yesheng`/`guangshu`/`ziyu`/`duwen`/`paotai`/`yuedong`/`baoxue`/`jidong`/`liangou`/`xiyang`/`feidan`/`zhiyuan`/`shenqiang`/`shihuo`/`mengji`/`yinbo`/`qianggu`）及 `standard_catalog` 标准技能。
   - 在 `decision34_u05_{ops,gujian_gwent,hearth,ow_xianjian,swd_cards,swd_chars}.rs` 将剩余无消费死旗标技能全部重做为真实上游牌局效果（摸牌/弃牌/回复/伤害/护甲/翻面/横置/置入武将牌/拼点），并更新 [`shahai_implemented_skills.rs`](../../../../crates/content-native-loader/src/shahai_implemented_skills.rs)（`IMPLEMENTED_U05_VM_SKILLS` 共 **919 项**已验证 VM 技能，消除 852 项未实现缺口）。
   - 在 [`intrinsic_named_ext.rs`](../../../../crates/engine-native-content/src/content/host/intrinsic_named_ext.rs) 与 [`play_card_targets.rs`](../../../../crates/engine-native/src/engine/play_card_targets.rs) 实装杀海拾遗专属基本牌、锦囊牌与轩辕剑/古剑机关牌（如 `yanjiadan_heart`、`yanjiadan_diamond`、`yanjiadan_club`、`yanjiadan_spade`、`lingjiandai`）的真实 `PlayCard` 目标校验与牌局结算。
2. **如实清点结果**：
   - [G0/U05 清单](../../../migration/g0/U05-杀海拾遗.md) 合计 **1,621** 项（332 将 / 1,007 技 / 275 卡 / 7 配置）：**`忠实实现 = 1,621 / 示意或自造 = 0 / 缺失 = 0`**，`U05` 在签署表与台账第①/⑧维达成 `✅`。

### 1.2 `L11 太虚幻境` 与 `L25 祖安武将`：清除 `subSkill` 正则伪影与分组兜底桶，嵌入预编译与已移植跨包技

1. **清除 `subSkill` 嵌套子键提取伪影**：
   - 从 [`taixu_skills_data.rs`](../../../../crates/content-native-loader/src/taixu_skills_data.rs) 删除 6 个 `subSkill` 正则伪影（`temp`、`buff`、`sub1`、`sub2`、`count`、`Use`）；从 [`zuan_data.rs`](../../../../crates/content-native-loader/src/zuan_data.rs) 删除 5 个 `subSkill` 正则伪影（`wuxie`、`equip`、`res`、`defend`、`zhou`）。
   - 在 [`decision34_inventory_builder.py`](../../../../scripts/legacy-harness/decision34_inventory_builder.py) 新增 `_strip_js_line_for_braces` 与 `_compute_subskill_line_mask`，屏蔽 `subSkill: { ... }` 嵌套块，全仓 37 份 G0 清单实现 `0` 项 `subSkill` 顶层误提。
2. **清除全部分组空桶并补齐显式分支与跨包技挂载**：
   - 在 [`decision34_l11_buffs.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_buffs.rs) 与 [`decision34_l11_skills.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_skills.rs) 删除全部 `l11_passive_active:*`、`l11_npc_*`、`l11_buff_draw:*`、`l11_buff_aura:*`、`l11_unhandled_skill:*` 分组桶以及合并 `ctx.start_draw(owner, 1)` / `discard_up_to_hand_or_equip` / `+1 hujia` 桶，逐技能补齐本包技/祝福效果与 [`d34_l11_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l11_mods.rs) 全部 13 项修饰旗标；从 `content/original/ported/*/*.npack.json` 提取非空预编译技能嵌入 [`packages/extensions/taixu-upstream-liuli/package.npack.json`](../../../../packages/extensions/taixu-upstream-liuli/package.npack.json)（**204** 项跨包主技 + **33** 项伴生子技 = **237** 条 `skill_index` / **232** 个字节码模块），并在 [`decision34_pack9.rs`](../../../../crates/content-native-loader/src/decision34_pack9.rs) 为 `("upstream-liuli", "taixu")` 挂载已移植跨包 NPC 技（`U05`/`U08`/`L19`/`L20`/`L27`）；在 [`taixu_campaign_data.rs`](../../../../crates/content-native-loader/src/taixu_campaign_data.rs) 将商铺池对齐真实 `buff_txhj_*` ID 并实装 `TaixuCampaignRun` 赛季图推进与快照往返。
   - **如实清点 `L11`**：[G0/L11 清单](../../../migration/g0/L11-太虚幻境.md) 合计 **936** 项（320 将 / 374 技 / 32 卡 / 206 节点 / 4 表面）：**`忠实实现 = 759 / 示意或自造 = 0 / 缺失 = 177`**（150 项未移植外部扩展 NPC 技能与 27 名关联 NPC 武将如实列为 `缺失`，签署表第①/⑧维记为 `部分`）。
   - 在 [`decision34_l25_rules.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_rules.rs) 与 [`decision34_l25_ssr.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_ssr.rs) 删除 `l25_unhandled_skill:*` 兜底，为 `L25_ZUAN_SKILLS` 全部剩余 40 项规则/语音/延时锦囊特效钩子补齐显式分支，并在 `intrinsic_named_ext.rs` 补齐 `ssr_yongsi_yuxi` 专属牌结算。[G0/L25 清单](../../../migration/g0/L25-祖安武将.md) 合计 **257** 项（60 将 / 84 技 / 5 卡 / 68 配置 / 40 钩子）达 **`忠实实现 = 257 / 示意或自造 = 0 / 缺失 = 0`**。

### 1.3 全仓修饰旗标与基础牌转化拆分接线（`U01/U05/U07/U08/L10/L11/L19/L20/L25/L27/L28/L29`）

- 将修饰求值与基础牌转化拆分为 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs)、[`d34_u05_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_u05_mods.rs)、[`d34_l11_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l11_mods.rs) 与 [`d34_view_as.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_view_as.rs)，并为 `dddqiahua`/`lzhangyi`/`lshengong` 补齐真实牌局动作，确保所有写入 `VmAux.flags` 的被动距离、范围、手牌上限、出杀次数、免伤与 `viewAs` 转化均在内核权威门控中实际生效。

### 1.4 牌桌 HUD、座位角标、手牌「理牌」与 5 个全屏模态窗挂载及真实浏览器测帧

1. **直接在真实牌桌组件树挂载**：
   - [`BattleShoushaHud.vue`](../../../../apps/web-client/src/components/battle/BattleShoushaHud.vue)：手杀阶段旗幡、读秒进度条、「将灯」身份徽章、如真似幻罗马品阶徽章及 5 个全屏弹窗入口按钮，直接挂载于 [`BattleBoard.vue`](../../../../apps/web-client/src/components/battle/BattleBoard.vue)。
   - [`SeatCard.vue`](../../../../apps/web-client/src/components/battle/SeatCard.vue)：座位十周年标记角标（如 `成略[阴]`、`暴戾[2]`）、`🍶酒态` 与 `L24` 攻击出框 `⚡GongJi`（支持 `flipX`）叠加显示。
   - [`HandStrip.vue`](../../../../apps/web-client/src/components/battle/HandStrip.vue)：手牌区右侧手杀「理牌」按钮（按牌类/花色/点数实时重排并同步手杀牌序状态）。
   - [`BattleModalsHost.vue`](../../../../apps/web-client/src/components/battle/modals/BattleModalsHost.vue) 统一挂载于 [`BattleView.vue`](../../../../apps/web-client/src/views/BattleView.vue)：
     - [`QhlyDetailModal.vue`](../../../../apps/web-client/src/components/battle/modals/QhlyDetailModal.vue)（千幻聆音武将立绘/台词/皮肤共享与套装切换全屏窗）
     - [`SkinSwitchModal.vue`](../../../../apps/web-client/src/components/battle/modals/SkinSwitchModal.vue)（皮肤切换藏珍阁、出框动作预览、坐标调参与 Spine 4.0 本地动皮导入窗）
     - [`MvpSettlementModal.vue`](../../../../apps/web-client/src/components/battle/modals/MvpSettlementModal.vue)（手杀 MVP 全场最佳结算大屏与献花/鸡蛋互动）
     - [`OmnisearchModal.vue`](../../../../apps/web-client/src/components/battle/modals/OmnisearchModal.vue)（全能搜索公开目录分页检索与暗牌/`eval` 拦截窗）
     - [`TaixuCampaignModal.vue`](../../../../apps/web-client/src/components/battle/modals/TaixuCampaignModal.vue)（太虚幻境赛季节点推进、九尾商铺真实 `buff_txhj_*` 购买与侍灵切换窗）

2. **真实无头 Chrome 渲染、交互、旧版对照与 60 帧帧率实测**：
   - 执行命令：`node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-10-round5-browser-surfaces`
   - 原始报告：[`decision34-browser-surfaces-report.json`](./decision34-browser-surfaces-report.json)（`passed: true`、`consoleErrors: []`）
   - 60 帧 `requestAnimationFrame` 帧间隔：`n = 60`、`p50_ms = 16.66`、`p95_ms = 16.67`、`p99_ms = 16.67`、`max_ms = 16.67`、**`approx_fps = 60`**
   - 实拍截图：
     - 牌桌顶栏 HUD、座位角标与手牌理牌：[`decision34-battle-table-hud.png`](./decision34-battle-table-hud.png)
     - 太虚幻境全屏战役与九尾商铺弹窗：[`decision34-modal-taixu-campaign.png`](./decision34-modal-taixu-campaign.png)
     - 旧版 `noname-liuli` 同尺寸静态样式对照基准：[`decision34-legacy-liuli-reference.png`](./decision34-legacy-liuli-reference.png)
     - 五套 UI/皮肤表面全幅截图：[`decision34-ui-skin-suites.png`](./decision34-ui-skin-suites.png)
     - 战役表面截图：[`decision34-taixu-campaign-surface.png`](./decision34-taixu-campaign-surface.png)
     - 扩展工作台全页截图：[`decision34-browser-surfaces.png`](./decision34-browser-surfaces.png)

## 2. 第五节门禁核验结果

| 门禁项 | 命令 | 结果 |
|---|---|---|
| 行数单测与源码行数门禁 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py' && python3 -B scripts/check-source-lines.py` | **PASS**（2 tests OK；`<= 2000` 总行、`<= 1000` Rust 有效行） |
| 原生三层依赖棘轮 | `python3 -B scripts/check-native-layering.py` | **PASS**（`0 upward references <= 0`） |
| 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | **PASS**（`2,229 passed; 0 failed; 6 ignored`） |
| 独立 `CARGO_TARGET_DIR` `-D warnings`（含 `wasm32`） | `CARGO_TARGET_DIR=/tmp/d34-r5-fix-werror RUSTFLAGS="-D warnings" cargo check --offline ... --all-targets` + `--target wasm32-unknown-unknown -p engine-native-wasm` | **PASS**（`0` 警告，已清理 `/tmp/d34-r5-fix-werror`） |
| Conformance 19 黄金场景 | `cargo run --offline -p conformance` | **PASS**（`passed=19, ALL PASS`） |
| 桌面端双路径测试（PA09） | `cargo test --offline -p desktop-client` | **PASS**（`20 passed; 0 failed; 1 ignored`） |
| 前端类型检查与单测 | `pnpm --dir apps/web-client typecheck && pnpm --dir apps/web-client test` | **PASS**（`vue-tsc` `0` 错；`489 passed; 0 failed`） |
| 真实 Headless Chrome 60 帧与弹窗截图 | `node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-10-round5-browser-surfaces` | **PASS**（`60.0 FPS`，`consoleErrors: []`） |
| 21 项仓库预检门禁 | `python3 -B scripts/release/preflight.py` | **PASS**（`21/21 全部通过`） |
| 裁决 25 Exact-Head CI（初版 `886fafafa`） | GitHub Actions Run `42771773936`（`refs/heads/native-convergence` = `886fafafa8a684fbbde807d2d1a8750ef78df073`） | **首次 9/10 PASS**（`perf-macos-latest` 共享 runner 微秒级抖动 `start-hash-p50 165/100 us`，见 [`exact-head-ci-run-42771773936-attempt1.json`](./exact-head-ci-run-42771773936-attempt1.json)）；**补跑 10/10 PASS**（见 [`exact-head-ci-run-42771773936.json`](./exact-head-ci-run-42771773936.json)） |

