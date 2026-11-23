# Batch U05-5b（`U05` 昆特牌 `gwent` 52 武将与 69 技能共 121 项等价）收口与验收证据（2026-11-23）

- **基线 HEAD**：`2150f8511556c8f6b90275e6b140388bc5cf3196`（Batch U05-5a 已发布，CI Run `45154024465` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-5b-gwent-chars`（`4a145b5f7..0ec2caac80d951b896bbaf7f9203c3af3b428e40`，三路应用补丁并保留 `main` 既有 `U05-2` / `U05-3` / `U05-6a` / `U05-6b` / `U05-5a` / `L11` / `L01` / `U01` 修复与 `5b` Round 2/3 全部修正）
- **独立复核裁决**：`verifier-u05-a` `#116 APPROVE`（审查 Round 2/3 全量修复、静态代码与上游对照以及 Chrome CDP 实跑探针：批准全部 **`121`** 项 `/character/gwent/` 武将与角色/衍生/全局技能升格为 `已复核等价`，`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（121 项升格 / 0 项保留）

- **升格为 `已复核等价`（`121` 项，全部 `reviewer="verifier-u05-a"`、`review_date="2026-11-23"`、`test_name="u05_b5b_gwent_chars_fixes + verifier-u05a-cdp-probe.mjs"`）**：
  - `52` 项 `U05:generals:*@.../character/gwent/character.js:*`（`gw_huoge`..`gw_saqiya` 全 52 名昆特牌武将）
  - `69` 项 `U05:skills:*@.../character/gwent/skill.js:*`（`sqlongyin`..`lanquan` 全 69 项角色技、衍生技与全局技能）
- **未动范围与保留边界**：
  - `U05:configs:gwent@.../main/config.js:39` 与 Batch U05-6b 的 `8` 项保留边界、Batch U05-1 的 `7` 项保留边界及其余未迁移子包继续保持不变；至此 `gwent` `/card/gwent/` 与 `/character/gwent/` 下共 `201` 项玩法/卡牌/武将/技能条目已全数升为 `已复核等价`（`0` 项玩法保留）。
- **最新计数**：`U05` **`927 / 694 / 0`**（`1,621` 项）；全局 **`3,247 / 1,177 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突处理、协同修复与模块拆分

- **词汇表与静态索引**：新增 `HostBehavior::U05GwentCharBatchA..=D`（tags `1190..=1193`，`HostBehavior::ALL: 633`），尾追加独立提示词表 [known_prompts_u05_5b.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts_u05_5b.rs)（`69` 条提示词，`0` 重名），接入 [u05_b5b_gwent/mod.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b5b_gwent/mod.rs) 及其 14 个子模块（`group_a..d` / `group_*_asks` / `group_*_steps` / `targets`）。
- **主线协同与回归修复**：
  - 保留 [core_ops_modifiers.rs](../../../../crates/engine-native-kernel/src/state/core_ops_modifiers.rs#L854-L867) `has_active_equip_kind`（`equip_card_has_kind && !is_gw_dieyi_masked`）并在 [targets.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b5b_gwent/targets.rs#L156-L157) `has_zhuge_clean` 统一调用，同时移除 [u05_b5a_gwent/mod.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b5a_gwent/mod.rs) 3 个未使用 `adventure_huoge` 重导出。
  - 保留 `main` `U05-5a` `extension_choices_dispatch.rs` 并末尾串接 [extension_choices_u05_5b.rs](../../../../crates/engine-native/src/ai/extension_choices_u05_5b.rs)（`plan_u05_5b_choice_action`）；协同合入 `hooks.rs` / `catalog/hooks.rs`（保留全量主线钩子并新增 `on_hujia_changed` / `on_phase_draw_before`）、`events/turn.rs` / `events/turn_reset.rs`、`combat_rules.rs`、`damage.rs`、`discard.rs`、`dying.rs`、`phase/draw.rs`、`intrinsic.rs`（`1,000` 有效行）与 `d34_flags.rs`。
  - 修复主线合入 `U05-2` `on_phase_end_gujian` 后的测试时序回归：[round3.rs](../../../../crates/engine-native/tests/u05_b5b_gwent_chars/round3.rs#L98-L102) 将 `grant_skill(&eg, &vg, "gj_food_molicha", 1)` 移至 `advance_to_seat_play(&mut eg, 2)` 之后（避免 `rounds=0` 夹具授予在推进经过 1 号座 `Phase::End` 时被 `food_tick` 清理），完整保留 `BOSS_REMEN`、`junchi_playable_cards` 与 `hupeng2` 对 `gw_dudayuanshuai2` 禁弃锁（同时护 `gwfutian`）。
  - 校准 [decision34_round4_u05_shahai.rs](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L537-L566) 剩余 `U05` VM 技能下界至 `>= 240`（实测 `245` 项 VM 技能、`925` 项 Host 技能、`1,372` 项合计技能）。
- **行数拆分保持 `<= 1,000` Rust 非测试有效行**：
  - 将 `lookup_standard_catalog_skill_zh` 自 [host_skill_bindings.rs](../../../../crates/content-native-loader/src/host_skill_bindings.rs#L11)（拆分后 `828` 有效行）拆出为独立模块 [host_skill_bindings_std.rs](../../../../crates/content-native-loader/src/host_skill_bindings_std.rs)（`213` 有效行）。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-5b/`）

- `cargo-test-round2-348b630a3.log` sha256 `f5c51388b049b7e5741a6813073a164c0b54b824871c7c11d4c005738ebca363`
- `probe-u05-5b.mjs` sha256 `08317a517d332f2829da00434569649fabbf2dfc87de91af6a0a312629f80609`
- `upstream-u05-5b-cdp-results.json` sha256 `f4a0a083d94439e2ffeddfc3af13148430a8afa4a965def11e9b958b518e922c`
- `verifier-u05a-cdp-probe.mjs` sha256 `96b7c9d43ce8cbcc50aa734ade5dd390e2badc874d36765515716efc1ccba49d`
- `verifier-u05a-cdp-results.json` sha256 `012a643ae0bb9a79b46b19bf47412528c41190af4651f5a92f70c54166a3309e`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（`9786` 文件 `<= 2000` 行，Rust 非测试源码 `<= 1000` 有效行）、`test_check_source_lines.py`（`2/2 OK`）与 `scripts/check-native-layering.py`（`0` 上行引用）通过；`RUSTFLAGS="-D warnings"` 原生（5 原生 crate `--all-targets`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块（`40` 非空套件）、`3,081` passed、`0` failed、`6` ignored（含 `g1_shahai` `362/0/0`〔含 `u05_b5b_gwent_chars_fixes` `26/0/0`〕、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块（`11` 非空套件）、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块（`2` 非空）、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块（`3` 非空）、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块（`146` 非空套件）、`8,388` passed、`0` failed、`8` ignored。
- `scripts/test_audit_upstream_skill_flags.py`：`6/6` tests OK。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`，`skill-vm` `ok=true`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
