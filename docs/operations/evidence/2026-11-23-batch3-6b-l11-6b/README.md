# Batch L11-6b（L11 太虚幻境 206 模式节点/侍从/局内效果包与净集 +35→554/936）独立复核与集成摘要

- **基线 HEAD**：`6ecddea03c8c2d5317da85780609de16f8306e70`（Batch L11-6a 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45074648377` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b6b-modes` `8a974b73f..b76098616`（`f1dea01e8`、`450a3cc85`、`793801760`、`b76098616`）
- **版本与内容锁**：规则 `333`／NGSA `107`／content schema `5`／ABI `759`／WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-l11`（`#100` **`APPROVE`**，含 Round 4 校验与三轮 Headless Chrome CDP）
- **清单变更**：全仓 `2,286 / 2,138 / 0` -> **`2,321 已复核等价 / 2,103 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `519 / 417 / 0` 升为 **`554 / 382 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与主线对齐修复

1. **核心钩子、缠怨门控与 `L11-6a` 共存协调**：
   - `catalog_queries.rs`、`catalog.rs`、`trigger_order.rs`、`apply_command.rs`、`legal_actions.rs`：保留主线 `L10` `has_rechanyuan`/`is_rechanyuan_sealed_def`/`is_rechanyuan_sealed_name`（含 `l10_rechanyuan` 旗标与 `hp <= 1` 保护濒死 `JI_JIU` 封禁），将其扩展同时识别 `"chanyuan"` 与 `"rechanyuan"`，并复用单一封禁实现暴露 `is_chanyuan_sealed` 与 `is_skill_chanyuan_blocked`（消除双调用重复）。
   - `kernel/sha/armor.rs`：保留主线 `L22` `_eloDraw` 偏好（`draw_judge_card_with_elo`）、`L11-4` 空牌堆早退及 `apply_huangtian_suit_rank`，并在判定花色处叠加 `effective_card_suit`（`hongyan`/`xinhongyan` 黑桃转红桃且受缠怨门控）。
   - `content/host/step/u08_batch_d_hooks.rs`：保留主线 `u01_fixes_3::u01_card_use_after_hook_3` 并协同 `l11_effect_pack::on_card_use_after_effect_pack`（含 `xinfu_sidao` 与 `xinfu_yisuan`）。
   - `archive/known_prompts_l11_b5.rs`：将新增 `"_txhj_skill_addEffectStyle:mulligan"` 从数组中部移至尾端（不改动既有索引），并在 `known_prompts_l11.rs` 尾端追加 13 项模式/效果包提示词。
   - `decision34_pack9.rs`、`judge_apply.rs`、`d34_flags.rs`、`eval.rs`、`d34_l11_mods.rs`、`d34_view_as.rs`、`core_ops.rs`、`core_ops_modifiers.rs`、`death.rs`：保留 `6ecddea03` 全量 `b6a` 修复与 `decision34_pack9_ui.rs` 拆分，完整并入 `kuanggu`/`hongyan`/`txhj_tanshi`/`chanyuan`/`xingshang`/`hengzheng`/`xinfu_tanbei`/`xinfu_yisuan`/`xinfu_sidao`（移出 VM/view-as 改挂 `Phase::Play` `useCardAfter`）、`lijue`/`guosi`/`zhangji`、`Equip1..6`、`taixu_mode/*` 与 `taixu_battle.rs`。

---

## 2. 独立复核 35 项升级与 171 项保留清单（`#100`）

- **升格为 `已复核等价`（35 项无外部阻塞依赖 `L11:modes:*`）**（`reviewer: "verifier-l11"`，`test_name: "l11_taixu_b6b"`，`review_date: "2026-11-23"`）：
  - **`extension_servant.js`（2 项）**：`servant`、`skillDesc`（Spine 播放元数据 `action` 维持 `简化近似`）。
  - **`event/event_universal.js`（18 项）**：`exam2`、`exam4017`/`4057`/`4058`/`4072`/`4080` 与 `txhj_fujiahaoli`..`txhj_tiandijinghua`（纯金币／体力／手牌／技能上限／已闭环 `store: true` 随机祝福，不涉 `L11_TAIXU_CARD_RANK`/`SKILL_RANK`）。
  - **DLC `extension_season.js`（15 项）**：`GFHuangJinZhiLuan`（2 项 `exam4017`）与 `GFQiQinMengHuo`（13 项纯 `Coin`/`MaxHp` `exam*` 节点）。
- **保持 `简化近似`（171 项 `L11:modes:*`）**：按 `#100` 与 Coordinator `#122` 严格保留 `taixuhuanjing`（1）、`extension_servant.js:action`（1）、依赖等级池/商店的 `event_universal.js`（50）与依赖 NPC 技能/未升格武将的 `extension_season.js`（119）共 171 项为 `简化近似`，待 Phase 3 闭合。
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b6b_modes.rs`、`l11_taixu_b6b_round3.rs`、`l11_taixu_b6b_round4.rs`（共 15 项经 `l_rounds` 运行，0 `debug_execute_effect`/`debug_resolve`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-6b/`）

- `dbe94b62b60b2b8892dd34e996d2bac3e85520622090dc67bd6d29537f6fe070` — `docs/operations/evidence-raw/review-l11-6b/verifier-probe-l11-6b-r4.mjs`
- `b2810185c986c617d1214da27d1b4217607d36f2b9a9ebf09ee1c896751f71da` — `docs/operations/evidence-raw/review-l11-6b/verifier-probe-l11-6b-r4.json`
- `db520a24c2cb94d043be03a01d2fcf761644270798e115ba8f12baa3f5b03583` — `docs/operations/evidence-raw/review-l11-6b/probe-l11-6b-r3.mjs`
- `50b9a435d68354ed53e97ec3a8d6595059476b18e9839121b0e8d070df938331` — `docs/operations/evidence-raw/review-l11-6b/probe-l11-6b-r3.json`
- `53d1585e1915a04789eaa5267968869a5cde9e8bd55e69a459ed9c1a01ef3cd2` — `docs/operations/evidence-raw/review-l11-6b/probe-l11-6b.mjs`
- `18a58248c8043b6381d3aff30ed589669707b588557c2ed310238096bb234537` — `docs/operations/evidence-raw/review-l11-6b/probe-l11-6b.json`

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`, `engine-native-wasm`, `content-native-loader`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`44` 组、**`2,897` passed / `0` failed / `6` ignored**（含 `l_rounds` **`292/0/0`**、`g1` **`395/0/1`**、`kernel` **`435/0/0`**、`engine-native-wasm` `26` 组 **`89/0/0`**）
   - `content-native-loader` offline：`12` 组、**`48` passed / `0` failed / `0` ignored**
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`（`start_4_42=8fe8316a...` / `start_2_1=6fb9d95a...` / `manifest_4_42=11214341...`）：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,204` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`recount_g0.py` 与 `preflight.py` 全过（提交前终验见发布日志）。
