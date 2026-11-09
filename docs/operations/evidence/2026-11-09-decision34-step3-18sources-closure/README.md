# 裁决 34 第三步证据：18 来源按上游 `file:line` 重做、正式包四端产品路径接入与 30/37 来源仓内闭环

- **核验日期**：2026-11-10（含对 `5824a3d9e..f5814401f` 的对抗复审与纠错）
- **适用分支**：`main` / `native-convergence`
- **对应任务条款**：《原生架构收敛执行方案》第三节第 3 项（「逐来源按上游重做：规则类（技能、卡牌、战役、模式）进原生规则或 rule-vm 包，并接入对应 `packages/extensions/*` 正式包与四端产品路径；每项测试须以上游代码描述的行为为预期（注释写上游 `file:line`），不得以原生实现反推断言。界面类（DOM/CSS 布局、动画、特效、WebGL）在原生前端实现，与上游同场景截图对照，并在真实浏览器记录帧率。按来源分批提交，每批更新扩展台账该来源行与清单状态。」）与第 6 项（「37 来源整包八项签署：第 1–3 项完成后，按扩展台账原八项逐来源复核并更新签署表；移动端目标平台待第 7 项。」）

---

## 一、本批按上游 `file:line` 完成忠实重做并接入正式包产品路径的 18 个来源

| 来源编号与名称 | 正式包 ID（`packages/extensions/*`） | 上游 `file:line` 依据 | 原生规则／前端实现落点 | 对应上游语义测试 |
| --- | --- | --- | --- | --- |
| `U01 3D精选` | `d3-upstream-noname` | `archive/upstream/noname/apps/core/extension/3D精选/character/{character.js:3-223, skill.js:6-6367}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_ext.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（31 名 `ddd_*` 武将与 64 项真实 `ddd*` 技能全部接入 `attach_formal_package_upstream_rules`，含 `dddjinggou`/`dddmoyan`/`dddbailei` 完整结算） | `crates/engine-native/tests/decision34_pack9_closure.rs::u01_d3_all_31_generals_and_64_ddd_skills_execute_and_roundtrip`、`formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `U04 金币` | `coin-upstream-noname` | `archive/upstream/noname/apps/core/extension/coin/extension.js:1-1354` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`dragBuyCoinShopItem`、`_buy`、Canvas 2D 天气粒子 `renderCoinCanvasParticles`、荣誉称号与富甲天下开关） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `U06 欢乐卡牌` | `huanle-upstream-noname` | `archive/upstream/noname/apps/core/extension/欢乐卡牌/{card/card.js:3-126, card/skill.js:3-145, extension.js:1-22}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（8 卡与 6 技：`monkey_peach`、`mianju`、`shoulijian`、`kuwu` 非横置守卫与弃牌/失体力双分支、`xuelunyang`、`jiuwei`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34_pack9_closure.rs::formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `U07 玩点论杀` | `wudian-upstream-noname` | `archive/upstream/noname/apps/core/extension/玩点论杀/{character/character.js:3-82, card/card.js:3-343, card/skill.js:3-38, character/skill.js:4-813}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_pack9.rs, decision34_ext.rs}`、`crates/content-native-loader/src/{decision34_closure.rs, decision34_pack9.rs, decision34_pack9_data.rs}`（9 将含刘馥/田豫/造之仁君/杨仪技能绑定、5 卡、30 技含 `g_youdishenru`/`ciqiu_destroy`/`fuchou3`/`diyjuntun2`/`juntun`/`juntun_global`/`liduan2` 全量接入正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::u07_wudian_jianbi_and_choudu_differential_and_save_restore`、`u07_wudian_eight`、`formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `U08 英雄杀` | `hero-kill-upstream-noname` | `archive/upstream/noname/apps/core/extension/英雄杀/{character/character.js:4-292, character/skill.js:4-3177, main/content.js:1-39}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_u08_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_closure.rs, decision34_pack9_data.rs}`（43 将、107 项 `yxs_*`/鲁班 `lshengong` 锻造置入装备区技能全量接入正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `L01 AI优化` | `ai-tuning-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9019-9292` | `crates/content-native-loader/src/ai_tuning.rs`（`ly_piliche`、`yiyi` 目标范围与收益评估）、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs`（`resolve_l01_yiyi_card` 接入正式包） | `crates/content-native-loader/tests/ai_tuning_loader.rs`（4/4 PASS）、`decision34_pack9_closure.rs` |
| `L05 金币` | `coin-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/coin/extension.js:1-1375` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（与 `U04` 同源：`dragBuyCoinShopItem`、`_buy`、Canvas 2D 天气粒子） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L06 五行生克` | `wuxing-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:1-229` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs`（`resolve_l06_wuxingpan_skill` 五行盘重铸属性切换，接入 `wuxing-upstream-liuli` 正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `L07 全能搜索` | `omnisearch-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/全能搜索/{extension.js:126-1091, SimplePagination.js:1-340}` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`SimplePagination` 分页搜索、关键词高亮、暗牌/牌堆门禁与 `eval` 禁用守卫） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L10 史诗卡牌` | `epic-cards-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/史诗卡牌/{extension.js:1-2975, cardPhantom.js:1-419, emoji.js:1-744}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs`（十常侍结党 `k_danggu` 真实授予 10 项 `scstaoluan..scsanruo` `SkillInstance` 与 `resolve_l10_scs_skill` 结算、殁亡 `k_mowang` 接入 `epic-cards-upstream-liuli` 正式包）、`extensionSurfaceRuntime.ts`（`triggerL10EmojiRebate`、`triggerL10ScsDieAudio`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34-extension-surfaces.test.mjs` |
| `L12 如真似幻` | `rzsh-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/如真似幻/{extension.js:12-650, js/dynamicSkin.js:1-120}` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`applyRzshDynamicSkinLighting` 伪 3D 光照透视与 Spine 4.0 净室动皮层） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L13 手杀MVP` | `shousha-mvp-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js:18-482` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`incrementMvpRecastCount` `_qy_chongzhu` 重铸统计、攻击/治疗/辅助 MVP 结算横幅与手牌上限角标） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L19 末日乱斗` | `doomsday-brawl-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-3411` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（真实 `jiangshifz` 男 5 血 5 技、`jiangshinj` 女 3 血 5 技与全部 66 项乱斗技能/钩子逐项结算接入正式包，零分组 `draw(1)` 桩） | `crates/engine-native/tests/decision34_pack9_closure.rs::l08_l14_l19_l20_complete_skills_triggers_and_save_restore`、`formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `L20 标记补充` | `mark-supplement-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:56-6505` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（3 张整肃牌、74 项标记技与钩子逐项按上游结算接入正式包，零分组 `draw(1)` / `_ => {}` 桩）、`extensionSurfaceRuntime.ts`（`applyL20MarkCssTheme`） | `crates/engine-native/tests/decision34_pack9_closure.rs::l08_l14_l19_l20_complete_skills_triggers_and_save_restore`、`decision34-extension-surfaces.test.mjs` |
| `L26 笨蛋插件` | `fool-plugin-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/笨蛋插件/extension.js:12-650` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`applyFoolCardHoverTheme` 笨蛋插件牌背花纹与出牌悬停光晕主题） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L27 选将美化` | `select-beautify-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:33-9090` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（全部 43 项选将扩展技能与钩子接入正式包，零分组 `draw(1)` 桩）、`extensionSurfaceRuntime.ts`（`applyL27SelectBeautifyTheme`） | `crates/engine-native/tests/decision34_pack9_closure.rs::l24_l25_l27_complete_skills_negative_guards_and_save_restore`、`decision34-extension-surfaces.test.mjs` |
| `L28 雷霆万钧` | `thunder-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3657-8968` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（14 名真实 `th_*` 武将、50 项真实主/子技能含 `th_tiansuan2_0..4` 五签、`th_pcaudio_*_card`、`th_zhengjing2`、`th_gushe2` 与 `th_taipingyaoshu` 接入正式包，清除 `_info`/`_map`/`_faq` 非技能键与 7 个虚构键） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34_pack9_closure.rs::formal_packages_extensions_load_and_execute_on_locked_product_path` |
| `L29 龙血玄黄` | `longxue-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/{extension.js:1, 专属技能区/手杀ui/skill/main1.js:1-1048}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（`mbcmqingzheng`/`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`/`mbweitong`/`mbcmxscw` 接入正式包）、`extensionSurfaceRuntime.ts`（`triggerLongxueHolySwordEffect`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34-extension-surfaces.test.mjs` |

---

## 二、对抗复审发现的 8 类问题与逐条修复记录（`5824a3d9e..f5814401f` 纠错）

1. **清除 `L19`、`L20`、`L27` 多模式分组 `ctx.start_draw(owner, 1)` 兜底与 `_ => {}` 空桩**：
   - 原 `decision34_u08_l19_l20_l27.rs` 将 `L19` 50 项技能合并为 `ctx.start_draw(owner, 1)`、将 `L20` 56 项技能/钩子合并为 `ctx.start_draw(owner, 1)` / `_ => {}`、将 `L27` 36 项技能/钩子合并为 `ctx.start_draw(owner, 1)`。现已拆分新增 `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l19_l20_l27.rs`，逐项按上游 `末日乱斗/extension.js`、`标记补充/extension.js`、`选将美化/extension.js` 的真实触发时机、标记计数、体力/护甲/手牌结算与负向守卫实现，零分组摸 1 桩。
2. **清除 `decision34_ext.rs` 残余 `_ => {}` 空分支与 `U08` `lshengong` 摸 1 简化**：
   - 补齐 `U01` `dddjinggou`/`dddmoyan`/`dddbailei`、`U06` `kuwu` 非横置目标守卫与弃牌/失体力双分支、`U07` `ciqiu_destroy`/`fuchou3`/`diyjuntun2`/`juntun`/`juntun_global`/`liduan2` 及 `U08` `lshengong` 从牌堆检索/锻造装备置入装备区逻辑。
3. **`L10` `k_danggu` 真实授予 10 项十常侍伴生 `SkillInstance`**：
   - 原 `k_danggu` 仅写整数存储旗标；现改为向角色真实授予 `L10_SCS_COMPANION_SKILLS`（`scstaoluan`、`scschuhai`、`scspojun`、`scsyanbi`、`scsjuece`、`scsfencheng`、`scsmieji`、`scsdanggu`、`scsniqu`、`scsanruo`）`SkillInstance`，并由 `resolve_l10_scs_skill` 按上游十常侍各侍技能结算。
4. **`U07` 补齐 `g_youdishenru` 与锁内容正式包武将技能绑定**：
   - `U07_WUDIAN_ALL_SKILLS` 补入 `"g_youdishenru"`（共 30 项），并在 `attach_formal_package_upstream_rules` 装载 `wudian-upstream-noname` 时将 `duoqi`、`youzhan`、`jianbi`、`choudu` 绑定至刘馥、田豫、造之仁君、杨仪（保持正式包 9 名武将列表不变，`u07_wudian_eight` 通过）。
5. **`L19` 修正丧尸武将性别/体力/技能组并补齐 8 项漏注技能，`L27` 补齐 6 项钩子**：
   - `jiangshifz` 修正为男、5 体力、`["baozou", "ganran", "shishi", "zaibian", "jinhua"]`；`jiangshinj` 修正为女、3 体力、`["xunmeng", "ganran", "shishi", "zaibian", "jinhua"]`；`L19_DOOMSDAY_SKILLS` 扩至 66 项，`L27_SELECT_BEAUTIFY_SKILLS` 扩至全部 43 项（31 技能 + 12 钩子）。
6. **`L28` 清除 `_info`/`_map`/`_faq` 非技能键与 7 个虚构键，补齐 16 项真实子技能**：
   - 从 `L28_THUNDER_EXTRA_SKILLS` 删除 `th_tiansuan_map`、`th_tiansuan_faq`、`th_tiansuan_faq_info`、`th_zhengjing2_info`、`th_pcaudio_info` 等 5 个文案/映射键与 7 个不在 `雷霆万钧/extension.js` 的虚构名，补齐 `th_tiansuan2_0..4` 五签、`th_pcaudio_*_card`、`th_zhengjing2`、`th_gushe2` 等 16 项真实子技能（39 项扩展技能 + 11 项武将主技 = 50 项真实技能）。
7. **`decision34_inventory_builder.py` 移除整包 `sid in (...)` 强行覆写，改为真实符号集与 `assets/config.json` 校验**：
   - 原脚本通过 `if sid in ("U04", "U05", ...): return "忠实实现"` 将 `U05` 缺失的 6 项配置误标为「忠实实现」（1,588 虚高 6 项），并将未闭合来源缺失的武将技能误标为「示意或自造」（647/2,200 失真）。现改为解析 `decision34_{pack9_data,closure,pack9}.rs` 真实注册符号集与 `packages/extensions/*/assets/config.json` 磁盘存在性，保持 `else: status = "缺失"`；重生成全部 `docs/migration/g0/*.md` 并消除 Section 1 与 Section 7 统计矛盾：
     - 总条目：**4,435 项**（834 武将、1,915 技能、380 卡牌、215 模式/战役节点、435 配置项、507 界面钩子、149 渲染面）
     - **忠实实现：1,582 项**
     - **示意或自造：456 项**（全部位于 7 个大体量未收口来源 `U05, L08, L09, L11, L14, L24, L25`）
     - **缺失：2,397 项**（全部位于上述 7 个大体量未收口来源）
     - **仓内 7 维 `✅` 忠实闭合来源：30/37**（`U01..U04, U06..U08, L01..L07, L10, L12, L13, L15..L23, L26..L29`，全部 0 示意、0 缺失）
     - **仓内 `部分` 来源：7/37**（`U05: 406/87/1128`、`L08: 44/52/310`、`L09: 85/12/5`、`L11: 0/4/938`、`L14: 48/88/5`、`L24: 14/32/0`、`L25: 70/181/11`）
     - **整包八项签署：0/37**（缺第⑥项移动端 Android/iOS 真机；`L10/L12/L28` 另待用户本机 4.0 动皮截图对照）
8. **修正本证据文档中原有的 10 处虚构包名、上游路径与测试函数名**：
   - 全部核对修正为 `packages/extensions/*` 真实包 ID（`d3-upstream-noname`、`huanle-upstream-noname`、`hero-kill-upstream-noname`、`epic-cards-upstream-liuli`、`shousha-mvp-upstream-liuli`、`fool-plugin-upstream-liuli`）、`archive/upstream/` 真实文件路径行号及 `formal_packages_extensions_load_and_execute_on_locked_product_path` 真实测试名。

---

## 三、第五节门禁实跑记录

1. **源码行数门禁与分层棘轮**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` -> `Ran 2 tests OK`
   - `python3 -B scripts/check-source-lines.py` -> 通过（全部受维护文件 `<= 2,000` 行，Rust 非测试文件 `<= 1,000` 有效行）
   - `python3 -B scripts/check-native-layering.py` -> `engine-native layering passed: 0 upward references <= 0`
2. **原生三 crate 全量、全工作区与跨目标测试**：
   - `cargo test --offline -p content-native-loader` -> `45 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p engine-native --test misc decision34_` -> `13 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p engine-native --test eight u07_wudian_eight` -> `1 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` -> **`2,220 passed; 0 failed; 6 ignored`**（剩余 6 个 `#[ignore]` 全为手工诊断探针）
   - `cargo test --offline --workspace --no-fail-fast` -> **`7,613 passed; 0 failed; 8 ignored`**
   - `cargo run --offline -p conformance` -> **`19/19 ALL PASS`**
   - `cargo test --offline -p desktop-client` -> **`20 passed; 0 failed; 1 ignored`**
   - `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` -> **`ALL CROSS-TARGET CHECKS PASSED`**
3. **独立 `CARGO_TARGET_DIR` Gate 1b `-D warnings` 与 `wasm32-unknown-unknown` 检查**：
   - `CARGO_TARGET_DIR=/tmp/noname-next-d34-review-gate1b RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && CARGO_TARGET_DIR=/tmp/noname-next-d34-review-gate1b RUSTFLAGS="-D warnings" cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` -> **exit 0（0 warnings）**，跑后已清理 `/tmp/noname-next-d34-review-gate1b`（裁决 23）。
4. **前端单测、Typecheck、Native 构建、Worker Smoke 与真实 Headless Chrome**：
   - `node --test apps/web-client/scripts/decision34-extension-surfaces.test.mjs apps/web-client/scripts/spine40-cleanroom.test.mjs` -> **`17 passed; 0 failed`**（`10/10` + `7/7`）
   - `pnpm --dir apps/web-client test` -> **`478 passed; 0 failed`**
   - `pnpm --dir apps/web-client run typecheck` -> **exit 0**
   - `pnpm --dir apps/web-client run build:native` -> **exit 0**
   - `pnpm --dir apps/web-client run smoke:native` -> **`83/83 PASS (failed: 0)`**
   - `node apps/web-client/scripts/decision34-browser-surfaces.mjs`（真实 Headless Chrome） -> `1160×773` DOM、`spine40Status: "Spine 4.0 净室 WebGL 活跃 · 12 三角形 · 动作 gongji"`、60 帧 **`p50 = 16.66 ms, p95 = 16.67 ms, p99 = 16.67 ms, approx_fps = 60.0 FPS, consoleErrors: []`**（见 [`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json) 与 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)）。
5. **`preflight.py` 21 项全量门禁**：
   - `python3 -B scripts/preflight.py` -> **`21/21 全部通过 ✅`**。
6. **按裁决 25 发布与 exact-head CI 实跑登记**：
   - 先前提交 `5824a3d9ebbcffcf5c796f4d4ba0e981d7c177ab`（`5824a3d9e`）对应 CI Run [`42641116293`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42641116293)（[`ci-run-42641116293-5824a3d9e.json`](ci-run-42641116293-5824a3d9e.json)）首跑 `10/10 PASS`；
   - 本轮对抗复审纠错代码提交 `1f0d16d6ca8c9d286488876162c6dea71ac1df59`（`1f0d16d6c`）通过单命令作用域 `pre-push` 守卫（`.git/hooks/pre-push` `sha256: e349af1e...` 零改动，远端 `main` 零改动）推送至 `refs/heads/native-convergence`；
   - Exact-head CI Run [`42668582089`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42668582089)（[`ci-run-42668582089-1f0d16d6c.json`](ci-run-42668582089-1f0d16d6c.json)）**首次运行（Attempt 1）10/10 全部通过 `conclusion: success`**（`native-web` `129338739506`、`desktop-macos-latest` `129338739981`、`build-macos-latest` `129338739997`、`perf-ubuntu-latest` `129338740011`、`gates-ubuntu` `129338740090`、`perf-macos-latest` `129338740097`、`desktop-ubuntu-latest` `129338740130`、`build-windows-latest` `129338740156`、`desktop-windows-latest` `129338740249`、`build-ubuntu-latest` `129338740310` 全绿）。
