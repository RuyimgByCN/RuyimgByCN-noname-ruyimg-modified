# Batch U05-7a（`U05` SWD 卡牌 Part 1：39 卡牌与 67 卡牌技能共 106 项等价）收口与验收证据（2026-11-23）

- **基线 HEAD**：`fb4a15c6b1fe0c6072c58e2be4445021a76449d0`（Batch U05-4 已发布，CI Run `45207312135` Attempt 1 因 `perf-macos-latest` 超时 `15m01s` 取消而 `failure`〔`logs-first.zip` sha256 `66f19d0a241ceadc413f2ca3d9f505a691f108e860350ba521de510754675205`〕，经 `--failed` 单次补跑后 Attempt 2 `10/10 success`〔`logs-attempt2.zip` sha256 `c737d1ac1230b32402b9b80c4969b905a342965211036c1619f77f1aead71a7c`〕；本批同步提交 [ci-runs.md](../ci-runs.md) 两条实测行）
- **合入来源**：`swarm/conv-u05-7a-swd-cards1`（`c727d3922..d94c0f0d5d100351ff44ea3fd095bec41a7c206e`，三路合入并保留 `main` 既有 `U05-1..6b` / `U05-4` R4 / `L11` / `L01` / `U01` 修复与 `U05-7a` Round 1–3 全部修正）
- **独立复核裁决**：`verifier-u05-b` `#56 APPROVE`（核验 Round 3 全 7 项修正——`qiankundai`、`g_shenmiguo`/`yuchankan`、`g_shencaojie`、`zhufangshenshi`/`g_zhufangshenshi`、`fuxiqin`、`g_baishouzhihu`、`response_kind_cards_core`——静态源码及 Chrome CDP 探针，批准全部 **`106`** 项 SWD 卡牌 Part 1 条目升格为 `已复核等价`，`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；`.git/config` 保持完整未改（sha256 `f862ba299c2bbd23c501ebeb0830f53caabc990a4da7e8005570470ff914bc25`）

## 清单收口（106 项升格 / 0 项保留）

- **升格为 `已复核等价`（`106` 项，全部 `reviewer="verifier-u05-b"`、`review_date="2026-11-23"`、`test_name="u05_b7a_swd_cards + verifier-probe-u05-7a.mjs"`）**：
  - `39` 项 `U05:cards:*@.../card/swd/card.js:*`（`baishouzhihu`..`zhuquezhizhang`，含全 8 张 `yuchan*`、`dujian`、`hufu`、`fuxiqin`、`pangufu`、`zhiluxiaohu` 等）
  - `67` 项 `U05:skills:*@.../card/swd/skill.js:*`（`qiankundai`、`g_hufu_{sha,shan,jiu}`、`g_yuchan_{swap,equip}`、`g_shencaojie`、`g_shenmiguo`、`g_zhufangshenshi`、`g_{baishouzhihu,cangchizhibi,huanglinzhicong,qinglongzhigui,xuanwuzhihuang,zhuquezhizhang}`、30 项 `*_equip1..5`、锻造/法宝/机关伴生技）
- **未动范围与保留边界**：
  - `U05` 其余未迁移子包（含 SWD Part 2 等，以及 `#56` 备忘归属 `u05-7b` 的 `Caoyao` / `kongxin` 运行时）、`U05:configs:gwent`、Batch U05-6b 的 `8` 项与 Batch U05-1 的 `7` 项继续保持不变。
- **最新计数**：`U05` **`1,189 / 432 / 0`**（`1,621` 项）；全局 **`3,509 / 915 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突处理、协同修复与门禁校准

- **词汇表与静态索引**：新增 `HostBehavior::U05SwdDujianCard..=U05SwdPassiveMarker`（tags `1280..=1287`，`HostBehavior::ALL: 646`），独立尾追加提示键（`known_prompts_u05_7a.rs` 新增 `13` 条 `u05_dujian_own_card`..`u05_baishouzhihu_forging_target`），接入 `swd_u05_7a*.rs`、`asks_swd.rs`、`extension_choices_u05_7a.rs` 并更新 [extension.rs](../../../../crates/engine-native-content/src/content/extension.rs#L93-L106) `InitHook::SwdJiqiHeld`（新增 `U05SwdHufuViewAs` 与 `U05SwdYuchanEquip`）。
- **主线 18 处碰撞合并与协同修复**：
  - [shahai_attach.rs](../../../../crates/content-native-loader/src/shahai_attach.rs) 保留主线 `off_deck`（`gjyuheng_*`/`yanjiadan_*`）与 `gw_dieyi_*`/`gjyuheng_*` 装备技能绑定，补齐 `fuxiqin -> ["kongxin"]` 并于尾部串联 `lookup_swd_u05_7a_behavior`；[combat_rules.rs](../../../../crates/engine-native-kernel/src/kernel/combat_rules.rs) 与 [sha/respond.rs](../../../../crates/engine-native-kernel/src/kernel/sha/respond.rs) / [sha/resume.rs](../../../../crates/engine-native-kernel/src/kernel/sha/resume.rs) 同时保留 `has_ow_mujing`、`blocks_hand_cards` 与 `yuchan_auto_view_as()` / `hufu_as` 响应转化；[play_card_targets.rs](../../../../crates/engine-native/src/engine/play_card_targets.rs) 保留 `Bingpotong` 自目标并接入 `CardKind::Dujian`。
  - [standard.rs](../../../../crates/engine-native-content/src/content/standard.rs#L639-L650)（`685` 行 / `515` 有效行）修复 `u05_b1_round2_fix06_g_chenhuodajie` 回归：仅对 `HostBehavior::U05SwdHufuViewAs` 允许 `(behavior, def.name)` 同行为多全球技（`g_hufu_sha`/`g_hufu_shan`/`g_hufu_jiu`），其余全局行为仍按 `behavior` 去重以防 `g_chenhuodajie` 与 `WD_CHENHUO_KEY` 双重入队。
  - [common.rs](../../../../crates/engine-native/tests/u05_b7a_swd_cards/common.rs#L260-L261) 修复 `u05_b7a_hufu_viewas_sha_shan_jiu_and_dying_with_save_restore`：在弃牌回手测试辅助函数 `remove_card_everywhere` 中同步清理陈旧 `declared_kinds`/`declared_slots`（与主线 `u05_b1_fixes.rs` 口径一致，同时保持弃牌堆内 `zhangba` 声明不被误清）。
  - 自 `intrinsic.rs`（降为 `1,057` 行 / `976` 有效行）拆出 `Dujian`/`Guisheqi`/`Xianluhui` 入 [swd_u05_7a.rs](../../../../crates/engine-native-content/src/content/host/step/swd_u05_7a.rs)（`675` 行 / `644` 有效行；连同 `swd_u05_7a_forging.rs` `356` 有效行、`swd_u05_7a_hufu.rs` `560` 有效行均 `<= 1,000` 有效行）。
  - 校准 [decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L537-L566) 剩余 VM 技能下界至 `>= 140`（入批前实测 `147` 项剩余 VM 技能；`host_skill_count >= 950` 与总技能 `>= 1,350` 保持通过）。
- **WebSocket 广播预算与阶段过滤门禁协同**：
  - [native_ws_e2e.rs](../../../../apps/game-server/src/native_ws_e2e.rs#L913-L917)：因 `U05-7a` 在 `InitHook::SwdJiqiHeld` 新增 4 项全局卡技（`g_hufu_{sha,shan,jiu}`、`g_yuchan_equip`）使双座 `shahai-swd` 对局每座公开技能由 `9` 升至 `13`，保留全部 301 条命令背压测试并将 `healthy_bytes` 上限自 `1 MiB` 调整为 `1.5 MiB`（`1536 * 1024`，实测 `healthy_bytes=1,198,758`、`buffered_outcomes=137`、`text_bytes=499,253`、`reconnect=pass`）。
  - 因 `SwdBaishouzhihuHeld` 在 [filtering/swd.rs](../../../../crates/engine-native-content/src/content/host/filtering/swd.rs#L405-L412)（补齐说明注释，总行 `1,101` / `901` 有效行以维持历史文档 `:1100` 行引用）改用 `EventData::CardsDiscarded` 对齐上游 `discardEnd`，同步重生成 [native-phase-filter-registry.tsv](../../../architecture/native-phase-filter-registry.tsv)（`39 -> 38` 条）并更新 [test_report_phase_filters.py](../../../../scripts/test_report_phase_filters.py)（`38` 条、`Discard: 2`）与 [test_report_phase_arms.py](../../../../scripts/test_report_phase_arms.py)（`28` 臂）。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-7a/`）

- `verifier-probe-u05-7a.mjs` sha256 `8073b5c90bef6296a5b3dd018405981f0c89f1929669728ee19ea04a51ab5efc`
- `verifier-u05-7a-cdp-results.json` sha256 `40be6237a1a197958f0ebcf1d135ff787003253783e83eb31e99417a56fbea10`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（所有维护文件 `<= 2,000` 行，Rust 非测试源码 `<= 1,000` 有效行）、`test_check_source_lines.py`、`scripts/check-native-layering.py`（`0` 上行引用）、`scripts/test_report_phase_filters.py`（`9/9 OK`）、`scripts/test_report_phase_arms.py`、`scripts/test_audit_upstream_skill_flags.py`（`6/6 OK`）与 `scripts/preflight.py`（`22/22` ✅）通过；`RUSTFLAGS="-D warnings"` 原生（5 原生 crate `--all-targets --no-run`）与 `wasm32-unknown-unknown`（`engine-native-wasm` `check`）零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44` 块（`40` 非空套件）、`3,120` passed、`0` failed、`6` ignored（含 `g1_shahai` `401/0/0`〔含 `u05_b7a_swd_cards` `18/0/0`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12` 块（`11` 非空套件）、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3` 块（`1` 非空）、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4` 块（`3` 非空）、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178` 块（`146` 非空套件）、`8,427` passed、`0` failed、`8` ignored。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build` `2.10s`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`，`skill-vm` `ok=true`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
