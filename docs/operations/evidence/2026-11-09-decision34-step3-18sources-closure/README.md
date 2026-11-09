# 裁决 34 第三步证据：18 来源按上游 `file:line` 重做、正式包四端产品路径接入与 30/37 来源仓内闭环

- **核验日期**：2026-11-09
- **适用分支**：`main` / `native-convergence`
- **对应任务条款**：《原生架构收敛执行方案》第三节第 3 项（「逐来源按上游重做：规则类（技能、卡牌、战役、模式）进原生规则或 rule-vm 包，并接入对应 `packages/extensions/*` 正式包与四端产品路径；每项测试须以上游代码描述的行为为预期（注释写上游 `file:line`），不得以原生实现反推断言。界面类（DOM/CSS 布局、动画、特效、WebGL）在原生前端实现，与上游同场景截图对照，并在真实浏览器记录帧率。按来源分批提交，每批更新扩展台账该来源行与清单状态。」）与第 6 项（「37 来源整包八项签署：第 1–3 项完成后，按扩展台账原八项逐来源复核并更新签署表；移动端目标平台待第 7 项。」）

---

## 一、本批按上游 `file:line` 完成忠实重做并接入正式包产品路径的 18 个来源

| 来源编号与名称 | 正式包 ID（`packages/extensions/*`） | 上游 `file:line` 依据 | 原生规则／前端实现落点 | 对应上游语义测试 |
| --- | --- | --- | --- | --- |
| `U01 3D精选` | `d3-select-upstream-noname` | `archive/upstream/noname/apps/core/extension/3D精选/character/{character.js:3-36, skill.js:6-3522}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_ext.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（31 名 `ddd_*` 武将与 63 项真实 `ddd*` 技能全部接入 `attach_formal_package_upstream_rules`） | `crates/engine-native/tests/decision34_pack9_closure.rs::u01_d3_all_31_generals_and_64_ddd_skills_execute_and_roundtrip`、`formal_packages_load_upstream_rules_on_product_path` |
| `U04 金币` | `coin-upstream-noname` | `archive/upstream/noname/apps/core/extension/金币/extension.js:6-711` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`dragBuyCoinShopItem`、`_buy`、Canvas 2D 天气粒子 `renderCoinCanvasParticles`、荣誉称号与富甲天下开关） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `U06 欢乐卡牌` | `joy-cards-upstream-noname` | `archive/upstream/noname/apps/core/extension/欢乐卡牌/{card.js:12-265, extension.js:10-270}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（8 卡与 6 技：`monkey_peach`、`mianju`、`shoulijian`、`kuwu`、`xuelunyang`、`jiuwei`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34_pack9_closure.rs::formal_packages_load_upstream_rules_on_product_path` |
| `U07 玩点论杀` | `wudian-upstream-noname` | `archive/upstream/noname/apps/core/extension/玩点论杀/{character.js:3-15, card.js:3-295, skill.js:4-455}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_pack9.rs, decision34_ext.rs}`、`crates/content-native-loader/src/{decision34_closure.rs, decision34_pack9.rs, decision34_pack9_data.rs}`（9 将、5 卡、29 技全量接入正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::u07_wudian_jianbi_and_choudu_differential_and_save_restore`、`formal_packages_load_upstream_rules_on_product_path` |
| `U08 英雄杀` | `yxs-upstream-noname` | `archive/upstream/noname/apps/core/extension/英雄杀/{character/character.js:4-46, character/skill.js:4-1505, card/card.js:4-49}` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_u08_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_closure.rs, decision34_pack9_data.rs}`（43 将、107 项 `yxs_*` 技能、3 卡全量接入正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::formal_packages_load_upstream_rules_on_product_path` |
| `L01 AI优化` | `ai-tuning-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9019-9292` | `crates/content-native-loader/src/ai_tuning.rs`（`ly_piliche`、`yiyi` 目标范围与收益评估）、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs`（`resolve_l01_yiyi_card` 接入正式包） | `crates/content-native-loader/tests/ai_tuning_loader.rs`（4/4 PASS）、`decision34_pack9_closure.rs` |
| `L05 金币` | `coin-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/金币/extension.js:6-711` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（与 `U04` 同源：`dragBuyCoinShopItem`、`_buy`、Canvas 2D 天气粒子） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L06 五行生克` | `wuxing-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/五行生克/extension.js:11-260` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_ext.rs`（`resolve_l06_wuxingpan_skill` 五行盘重铸属性切换，接入 `wuxing-upstream-liuli` 正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::formal_packages_load_upstream_rules_on_product_path` |
| `L07 全能搜索` | `omnisearch-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js:126-1091` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`SimplePagination` 分页搜索、关键词高亮、暗牌/牌堆门禁与 `eval` 禁用守卫） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L10 极略` | `jilue-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/极略/extension.js:24470-26455` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs`（十常侍结党/殁亡及 10 侍伴生技接入 `jilue-upstream-liuli` 正式包）、`extensionSurfaceRuntime.ts`（`triggerL10EmojiRebate`、`triggerL10ScsDieAudio`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34-extension-surfaces.test.mjs` |
| `L12 如真似幻` | `rzsh-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js:12-650` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`applyRzshDynamicSkinLighting` 伪 3D 光照透视与 Spine 4.0 净室动皮层） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L13 最佳Mvp` | `best-mvp-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/最佳Mvp/extension.js:18-482` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`incrementMvpRecastCount` `_qy_chongzhu` 重铸统计、攻击/治疗/辅助 MVP 结算横幅与手牌上限角标） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L19 末日乱斗` | `doomsday-brawl-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:331-3519` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_u08_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（2 将与 58 项真实乱斗技能接入正式包） | `crates/engine-native/tests/decision34_pack9_closure.rs::l08_l14_l19_l20_complete_skills_triggers_and_save_restore`、`formal_packages_load_upstream_rules_on_product_path` |
| `L20 标记补充` | `mark-supplement-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:56-6505` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_u08_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（3 张整肃牌、42 标记技与 39 钩子接入正式包）、`extensionSurfaceRuntime.ts`（`applyL20MarkCssTheme`） | `crates/engine-native/tests/decision34_pack9_closure.rs::l08_l14_l19_l20_complete_skills_triggers_and_save_restore`、`decision34-extension-surfaces.test.mjs` |
| `L26 愚人卡牌` | `fool-cards-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/愚人卡牌/extension.js:12-650` | `apps/web-client/src/engine/extensionSurfaceRuntime.ts`（`applyFoolCardHoverTheme` 愚人卡牌悬停光晕与背牌花纹主题） | `apps/web-client/scripts/decision34-extension-surfaces.test.mjs` |
| `L27 选将美化` | `select-beautify-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/选将美化/extension.js:33-539` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34_pack9.rs, decision34_u08_l19_l20_l27.rs}`、`crates/content-native-loader/src/{decision34_pack9.rs, decision34_pack9_data.rs}`（39 项选将扩展技能接入正式包）、`extensionSurfaceRuntime.ts`（`applyL27SelectBeautifyTheme`） | `crates/engine-native/tests/decision34_pack9_closure.rs::l24_l25_l27_complete_skills_negative_guards_and_save_restore`、`decision34-extension-surfaces.test.mjs` |
| `L28 雷霆万钧` | `thunder-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js:3657-8968` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（14 名真实 `th_*` 武将、35 项真实主技能与 `th_taipingyaoshu` 接入正式包） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34_pack9_closure.rs::formal_packages_load_upstream_rules_on_product_path` |
| `L29 龙血玄黄` | `longxue-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:32-449` | `crates/engine-native-kernel/src/rvm_ops/commit_up37/{decision34.rs, decision34_ext.rs}`、`crates/content-native-loader/src/decision34_closure.rs`（`mbcmqingzheng`/`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`/`mbweitong` 接入正式包）、`extensionSurfaceRuntime.ts`（`triggerLongxueHolySwordEffect`） | `crates/engine-native/tests/decision34_rules_and_campaigns.rs`、`decision34-extension-surfaces.test.mjs` |

---

## 二、37 来源 G0 清单去噪与当前真实统计（4,435 项）

- `scripts/legacy-harness/decision34_inventory_builder.py`：
  - 保持分类器 `else:` 默认分支为未闭合（缺失／示意），绝不将未实现条目默认标为「忠实实现」；
  - 过滤 `U06`、`U07`、`L10`、`L28` 中因正则缩进误匹配进顶层技能表的 `content` 子步骤（`_*_1`、`*_backup`、`*_viewas` 等内部临时对象）；
  - 全量重生成 `docs/migration/g0/README.md` 与 `docs/migration/g0/{U01..U08,L01..L29}-*.md`：
    - 总条目：**4,435 项**（834 武将、1,915 技能、380 卡牌、215 模式/战役节点、435 配置项、507 界面钩子、149 渲染面）
    - **忠实实现：1,588 项**
    - **示意或自造：647 项**（全部位于 7 个大体量未收口来源 `U05, L08, L09, L11, L14, L24, L25`）
    - **缺失：2,200 项**（全部位于上述 7 个大体量未收口来源）
    - **仓内 7 维 `✅` 忠实闭合来源：30/37**（`U01..U04, U06..U08, L01..L07, L10, L12, L13, L15..L23, L26..L29`）
    - **仓内 `部分` 来源：7/37**（`U05, L08, L09, L11, L14, L24, L25`）
    - **整包八项签署：0/37**（缺第⑥项移动端 Android/iOS 真机；`L10/L12/L28` 另待用户本机 4.0 动皮截图对照）

---

## 三、第五节门禁实跑记录

1. **源码行数门禁与分层棘轮**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` -> `Ran 2 tests in 0.010s OK`
   - `python3 -B scripts/check-source-lines.py` -> `Source line limit passed: 9209 files <= 2000 lines; Rust non-test files <= 1000 effective lines`
   - `python3 -B scripts/check-native-layering.py` -> `engine-native layering passed: 0 upward references <= 0`
2. **原生三 crate 全量、全工作区与跨目标测试**：
   - `cargo test --offline -p content-native-loader --test ai_tuning_loader` -> `4 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p engine-native --test misc decision34` -> `13 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p game-server --test native_product_path` -> `10 passed; 0 failed; 0 ignored`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` -> **`2,220 passed; 0 failed; 6 ignored`**（剩余 6 个 `#[ignore]` 全为手工诊断探针）
   - `cargo test --offline --workspace --no-fail-fast`（177 二进制） -> **`7,613 passed; 0 failed; 8 ignored`**
   - `cargo run --offline -p conformance` -> **`19/19 ALL PASS`**
   - `cargo test --offline -p desktop-client` -> **`20 passed; 0 failed; 1 ignored`**
   - `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` -> **`ALL CROSS-TARGET CHECKS PASSED`**
3. **独立 `CARGO_TARGET_DIR` Gate 1b `-D warnings` 与 `wasm32-unknown-unknown` 检查**：
   - `CARGO_TARGET_DIR=/tmp/noname-next-d34-gate1b RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && CARGO_TARGET_DIR=/tmp/noname-next-d34-gate1b RUSTFLAGS="-D warnings" cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` -> **exit 0（0 warnings）**，跑后已清理 `/tmp/noname-next-d34-gate1b`（裁决 23）。
4. **前端单测、Typecheck、Native 构建、Worker Smoke 与真实 Headless Chrome**：
   - `node --test apps/web-client/scripts/decision34-extension-surfaces.test.mjs apps/web-client/scripts/spine40-cleanroom.test.mjs` -> **`17 passed; 0 failed`**（`10/10` + `7/7`）
   - `pnpm --dir apps/web-client test` -> **`478 passed; 0 failed`**
   - `pnpm --dir apps/web-client run typecheck` -> **exit 0**
   - `pnpm --dir apps/web-client run build:native` -> **exit 0**
   - `pnpm --dir apps/web-client run smoke:native` -> **`83/83 PASS (failed: 0)`**
   - `node apps/web-client/scripts/decision34-browser-surfaces.mjs`（真实 Headless Chrome） -> `1160×773` DOM、`spine40Status: "Spine 4.0 净室 WebGL 活跃 · 12 三角形 · 动作 gongji"`、60 帧 **`p50 = 16.66 ms, p95 = 16.67 ms, p99 = 16.67 ms, approx_fps = 60.0 FPS, consoleErrors: []`**（见 [`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json) 与 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)）。
5. **`preflight.py` 21 项全量门禁**：
   - `python3 -B scripts/preflight.py` -> **`21/21 全部通过 ✅`**。
