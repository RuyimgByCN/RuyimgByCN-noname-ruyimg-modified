# Batch U05-3（`U05` 守望先锋 `ow` 127 项）收口与验收证据（2026-11-23）

- **基线 HEAD**：`35bc3def969bf9c4efa1a9c1806c19be11513a01`（Batch U05-2 已发布，CI Run `45107904173` Attempt 1 `10/10 success`；同步纳入其未提交的 `ci-runs.md` 实测行）
- **合入来源**：`swarm/conv-u05-3-ow`（`8a879f8e4..53ed4b1fa32996619203e330c82417c154172dd2`，`127` 项候选范围）
- **独立复核裁决**：`verifier-u05-a` `#113 APPROVE`（审查 `53ed4b1fa` Round 1–3 修复后全量 `127` 项等价、`0` 项保留 `简化近似`）
- **版本不变量（未修改）**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动

## 清单收口（127 项全量升格 / 0 项保留）

- **升格为 `已复核等价`（`127` 项，全部 `reviewer="verifier-u05-a"`、`review_date="2026-11-23"`、`test_name="u05_b3_ow_fixes + verifier-u05a-cdp-probe.mjs"`）**：
  - `24` 名武将（`character/ow/character.js`：`ow_orisa`、`ow_wensidun`、`ow_baolei`、`ow_zhaliya`、`ow_luba`、`ow_heiying`、`ow_laiyinhate`、`ow_tuobiang`、`ow_banzang`、`ow_ana`、`ow_kuangshu`、`ow_maikelei`、`ow_mei`、`ow_heibaihe`、`ow_dva`、`ow_sishen`、`ow_shibing`、`ow_yuanshi`、`ow_chanyata`、`ow_falaozhiying`、`ow_luxiao`、`ow_zhixuzhiguang`、`ow_tianshi`、`ow_liekong`）
  - `103` 个技能（`character/ow/skill.js`，协同 `6` 项派生子技/共用技共 `109` 项原生映射于 `OW_HOST_SKILL_MAP`：`HostBehavior` tags `1040..=1043` `U05OwBatchA..=D`，含 `yihun2`、`ow_target_enabled` / `eval_card_target_canceled_opt`、`huoyu`/`dianji`、`shanxian` 翻面劫持与首座轮次步进、`bfengshi`/`bfengshi3`、`lianpo` 可选确认、`feiren3` 与离线补给牌池）
- **最新计数**：`U05` **`351 / 1,270 / 0`**（`1,621` 项）；全局 **`2,671 / 1,753 / 0`**（`4,424` 项，`缺失: 0`）；整包 8 维签字仍 **`0/37`**。详见 [inventory_status.json](../../../migration/g0/inventory_status.json) 与 [U05-杀海拾遗.md](../../../migration/g0/U05-杀海拾遗.md)。

## 主线合入冲突、行数拆分与集成修复

- **3-way 合并与模块拆分**：保留 `HEAD` `U01-3`/`L11`/`U05-1`/`U05-2` 共存接口并合入 `U05-3`（`HostBehavior::ALL: 615`、`known_prompts_u05_3.rs` 尾追加 `67` 条提示词、`catalog/hooks.rs`、`machine/helpers.rs`、`content/host/skill_defs.rs`、`u08_batch_d_hooks.rs` 兼保 `fid` 签名与 `host_card_use_after_ow`、`d34_flags.rs` 透传 `Option<CardId>`、`lian_ying.rs` 兼具 `from != to` 与 `l11_discard_non_hand`、`combat_rules.rs` / `sha/respond.rs` / `shapes.rs` 兼具 `is_u01_hand_locked` 与 `blocks_hand_cards`/`blocks_card_use_and_save`）。将 `crates/engine-native-kernel/src/kernel/events/turn.rs` 拆出 `crates/engine-native-kernel/src/kernel/events/turn_reset.rs`（保持全部 Rust 非测试源码 `<= 1,000` 有效行）。
- **集成回归修复（无色 AOE 指向过滤）**：`HEAD` 在 `crates/engine-native-kernel/src/kernel/aoe.rs` 中由 `AoeResolveMachine` 调用 `eval_card_target_canceled` 时将虚拟 `card: 0` 直接作实体卡传入，导致无色虚拟 `Chiyuxi`（`huoyu`）误读 `cards[0]` 黑桃属性被 `weimu` 错误格挡（`r3_defect2_huoyu_colorless_and_dianji_preserves_suit_against_black_wards` 失败）。已改由 `eval_card_target_canceled_opt` 仅在 `ctx.core.table.contains(&self.card)` 时传 `Some(self.card)`，虚拟无牌 AOE 传 `None` 并按 `nature`/`need` 推演 `Chiyuxi`/`Jingleishan`，合入全量通过。
- **`decision34_round4_u05_shahai.rs` 校准**：`ow` 技能迁入 `U05OwBatchA..=D` 后，对应将 Catalog 剩余 VM 技能数下界由 `>= 600` 校准为 `>= 500`（总技能数仍 `>= 900`）。

## 独立 CDP 与作者原始证据指纹（`docs/operations/evidence-raw/review-u05-3/`）

- `probe-u05-3.mjs` sha256 `9f4ef08845fc4722beff6136c6ca1b1ab5dd4a266ab44265abc3dcd652ebce81`
- `upstream-u05-3-cdp-results.json` sha256 `8c783138b87a03e2771551be4846631a08184f45b88b190e0a4087592d1d0220`
- `probe-u05-3-round2.mjs` sha256 `841b74bde7262a8881f0d54a8e8be525502acb28dffaba301d1ea84c28d5e874`
- `upstream-u05-3-round2-cdp-results.json` sha256 `fc00d671d22b0287e54accd41a828a96b9037f8a058684e45184f45b88b190e0`
- `verifier-u05a-cdp-probe.mjs` sha256 `b705a320e111e09e08d7a990dd51f3aeaf6dd5fee2bf681e8adc852cb6cc90ba`
- `verifier-u05a-cdp-results.json` sha256 `aa19edb0b13ed8a611400528b6fb02ae4d13c2fb5ee515578d77d5fb773a0b1a`

## 主线最终合入验证结果（最后一次代码修改后全量实测）

- 静态与严厉编译：`scripts/check-source-lines.py` 与 `scripts/check-native-layering.py` 通过；`RUSTFLAGS="-D warnings"` 原生（`engine-native-kernel` / `engine-native-content` / `engine-native --all-targets` / `engine-native-wasm` / `content-native-loader`）与 `wasm32-unknown-unknown` 零警告通过。
- `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm --no-fail-fast`：`44/44` 块、`2,984` passed、`0` failed、`6` ignored（含 `g1_shahai` `265/0/0`、`u05_b3_ow_fixes` `31/0/0`、`kernel` `436/0/0`、`g1` `395/0/1`、`l_rounds` `292/0/0`、`nc_rounds` `311/0/5`）。
- `cargo test --offline -p content-native-loader`：`12/12` 块、`48` passed、`0` failed。
- `cargo run --offline -p conformance`：`19/19` (`ALL PASS`)；`cargo test --offline -p conformance`：`3/3` 块、`4` passed、`0` failed。
- `cargo test --offline -p desktop-client`：`4/4` 块、`20` passed、`0` failed、`1` ignored。
- `cargo test --offline --workspace --no-fail-fast`：`178/178` 块、`8,291` passed、`0` failed、`8` ignored。
- `apps/web-client`：`typecheck` 通过、`test` `540/540` pass、`build:native` 通过、`smoke:native` `83/83` PASS（`failed: 0`、`sources=37/37`、`packages=46/46`）、`build`（`244` 模块 `vite build`）通过并还原 `apps/web-client/dist/`。
- 跨端一致性：`scripts/check-native-wasm-cross-target.py` `native_wasm_match=true`（标准 40 步 `6a59d996801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、欢乐包 20 步 `42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`）；`scripts/cross-target-hash.sh` `ALL CROSS-TARGET CHECKS PASSED`（`start_4_42=wasm_start_4_42=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`，`start_2_1=wasm_start_2_1=6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`，`manifest_4_42=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`）。
- 详见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md) 与 [native-progress.md](../../../architecture/native-progress.md)
