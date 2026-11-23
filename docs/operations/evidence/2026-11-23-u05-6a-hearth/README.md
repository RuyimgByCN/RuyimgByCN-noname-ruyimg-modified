# Batch U05-6a（`U05` 炉石传说 `hearth` 167 项）收口与验收证据（2026-11-23）

- **基线 HEAD**：`a2e340f868c62b161a73b1f21907233b7eda4234`（Batch U05-3 已发布，CI Run `45116138360` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-6a-hearth`（`b7ac33cb6d6164be6b4f3e124c24c668e666cac6..9703e2188159f6cb5d204c5890c12cfb5e4447a0`，`167` 项候选范围）
- **独立复核裁决**：`verifier-u05-b` `#27 APPROVE`（审查 `9703e2188` Round 1–3 修复后全量 `167` 项等价、`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（167 项全量升格 / 0 项保留）

- **升格为 `已复核等价`（`167` 项，全部 `reviewer="verifier-u05-b"`、`review_date="2026-11-23"`、`test_name="u05_b6a_hearth_fixes + verifier-probe-u05-6a.mjs"`）**：
  - `57` 张炉石卡牌（`card/hearth/card.js` `13` 张 + `character/hearth/card.js` `44` 张衍生牌：`chuansongmen`、`dunpaigedang`、`shandianjian`、`shihuawuqi` 及 `hsbaowu_*` / `hsdusu_*` / `hsfashu_*` / `hsjixie_*` / `hslingjian_*` / `hsmengjing_*` / `hsqingyu_*` / `hsqizhou_*` / `hsshenqi_*` / `hstianqi_*` / `hszuzhou_*` / `tuteng1..4`）
  - `110` 项炉石技能（`card/hearth/skill.js` + `character/hearth/skill.js`，协同衍生/伴生条共 `119` 项原生映射于 `HEARTH_HOST_SKILL_MAP`：`HostBehavior` tags `1220..=1225` `U05HearthCards` / `U05HearthBatchA..=E`）
- **最新计数**：`U05` **`518 / 1,103 / 0`**（`1,621` 项）；全局 **`2,838 / 1,586 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突、模块拆分与集成修复

- **3-way 合并与模块拆分**：保留 `HEAD` 全部 `L11` / `U01-3` / `U05-1..3` 接口并合入 `U05-6a`（`HostBehavior::ALL: 621`、独立尾追加 `known_prompts_u05_6a.rs` `79` 条提示词、`catalog_queries.rs` 沿用 `resolved_card_kind` 与统一 `is_locked_skill_name` / `hs_fengyin_xuetu` / `hs_qianfu2_active`、拆出 `play_card_targets_hearth.rs` 协同 `play_card_targets.rs` 与 `play_card.rs`、`content/hooks.rs` + `content/host.rs` 复合 `runs_before_phase_begin` / `host_on_outer_settled` / `host_on_phase_end` / `host_on_player_died(killer)`、`d34_flags.rs` + `d34_u05_mods.rs` 并行接入 `qianfu2` 与 `hearth` 修正）。
- **集成回归修复（`stone_chenshui` 离线梦境牌检索）**：`shahai_u05_hearth.rs` 将 `is_hearth_derived_card_name`（含全部 5 张 `hsmengjing_*`）标为 `off_deck` 随局入 `detached_cards`，导致 Batch U05-1 的 `step_stone_chenshui`（[u05_b1_zhenfa.rs](../../../../crates/engine-native-content/src/content/host/step/u05_b1_zhenfa.rs)）仅查 `deck`/`discard` 未授予梦境牌（`u05_b1_main_config_dependencies_wuxing_rate_and_stone_chenshui` 失败）。已同步使 `step_stone_chenshui` 优先从 `detached_cards` 抽取并回退 `deck`/`discard`，并更新 `u05_b1_fixes.rs::total_tracked_cards` 计入 `detached_cards` 与 `off_court` 守恒。
- **`decision34_round4_u05_shahai.rs` 校准**：`hearth` 迁入原生 `HostBehavior` 后，剩余 `vm_skill_names` 下界设为 `>= 400` 且校验 `host_skill_count >= 100`、总 `catalog.skills.len() >= 900`。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-6a/`）

- `probe-u05-6a.mjs` sha256 `86455519d7ee269f887271cb37232d2472722e2145526fc230bc686f26209b53`
- `upstream-u05-6a-cdp-results.json` sha256 `7360fcdbf23f5dd2299288bad9035c66c23a2ccda79e2b52dc8f57774548e82f`
- `verifier-probe-u05-6a.mjs` sha256 `18f9896c1ff584ef38d1dd75b4ab514e98a99296be95c53aaeabd41d9feb1b6c`
- `verifier-u05-6a-cdp-results.json` sha256 `b6642cd3211c7c0df4b70ab2ac89ac749f8bddb8d94473b42bc8b5f32c8c8507`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py`（`9716` 文件）与 `scripts/check-native-layering.py`（`0` 上行引用）通过；`RUSTFLAGS="-D warnings"` 原生（`engine-native-kernel` / `engine-native-content` / `engine-native --all-targets` / `engine-native-wasm` / `content-native-loader`）与 `wasm32-unknown-unknown` 零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块、`3,010` passed、`0` failed、`6` ignored（含 `g1_shahai` `291/0/0`、`u05_b6a_hearth_fixes` `26/0/0`、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块、`8,317` passed、`0` failed、`8` ignored。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`wasm-pack` 部署 `native-pkg`/`wasm-pkg` 后 `smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`engine-cli` / `wasm-hash.mjs` 跨端校验 `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)。
