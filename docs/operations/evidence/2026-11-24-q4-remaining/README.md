# 2026-11-24 Batch Q4-remaining 合入证据（Phase 2 收口：`L03/L04/L15/L17/L21/L26/U02/U03/U04`）

- **日期与基线**：`2026-11-24`；在 `8fa7589963bbbe724dc89e656ffcdf6b1afdd8ea`（Batch `U05-7d`）上合入 `swarm/conv-q4-remaining`（`586b51cdc..0e06ca047480eefdf6b8c86bb307f3702395079e`，`verifier-l11` `#114 APPROVE`）。
- **冻结边界**：`RULES_VERSION=333`、`NGSA=107`、content schema `5`、ABI `759`、WS `8`、14 条 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 均未改动；整包 8 维签收维持 `0/37`。

## 1. 主要代码与模块拆分

- **词汇与提示词**：接入 `HostBehavior::Q4BossPlaySkill`（`1387`）与 `HostBehavior::Q4FoolSkill`（`1388`），`HostBehavior::ALL = 701`（[metadata.rs](../../../../crates/engine-native-kernel/src/vocabulary/host/metadata.rs) 中二者元数据为 `None`）；新增 [known_prompts_q4.rs](../../../../crates/engine-native-kernel/src/archive/known_prompts_q4.rs#L3-L37) `33` 条提示词。
- **行数门禁拆分**：
  - 将 `has_active_wansha` / `wansha_blocks_rescue` 拆入 [catalog_queries_q4.rs](../../../../crates/engine-native-kernel/src/catalog_queries_q4.rs)（于 [catalog.rs](../../../../crates/engine-native-kernel/src/catalog.rs#L1325) 声明）。
  - 将 `is_same_team_or_friend` / `is_hostile_to` 拆入 [core_ops_teams.rs](../../../../crates/engine-native-kernel/src/state/core_ops_teams.rs)（于 [state.rs](../../../../crates/engine-native-kernel/src/state.rs#L16) 声明），[core_ops.rs](../../../../crates/engine-native-kernel/src/state/core_ops.rs) 保留 `pub fn jun_group`。
  - 将 `append_q4_formal_rules` 拆入 [q4_engine_bridge.rs](../../../../crates/content-native-loader/src/q4_engine_bridge.rs)，于 [decision34_pack9.rs](../../../../crates/content-native-loader/src/decision34_pack9.rs#L1002-L1008) 委托调用。
- **主线合入与语义修复**：
  - [events.rs](../../../../crates/engine-native-kernel/src/kernel/events.rs) 保留 `HEAD` 出牌阶段 `Sha` 逻辑（未回引已删除的 `Bingheyaosai`）并接入 `jintao` `q4_play_sha_count`。
  - [machine.rs](../../../../crates/engine-native-kernel/src/machine.rs) 在 `reshuffle_discard_into_deck` 内统一调用 `cleanup_discard_to_deck`、`elo_on_wash_deck` 与 `on_deck_washed`，并统一判定牌摸牌走该路径。
  - [cleanup.rs](../../../../crates/engine-native-kernel/src/resolution/cleanup.rs) 中 `unwind_to_phase_loop` 将弹出帧清理委托至 `frame_hooks::cleanup_unwound_frame`（消除重复逻辑）。
  - Round 4 接入 `retieji` `OrdinaryJudge`、独立 `L03`/`U02` boss 分发（含 `boss_baonu` / `boss_baonuwash`），[boss_play_data.rs](../../../../crates/content-native-loader/src/boss_play_data.rs#L31-L33) 维持 `U02_BOSS_GENERALS`（15 将）、`L03_BOSS_GENERALS`（8 将）及 `boss_lvbu1..3` 在四势力外统一映射 `Faction::Qun` 的边界记录（见 [native-rules-baseline.md](../../../architecture/native-rules-baseline.md)）。
  - 集成回归入口 [l_rounds.rs](../../../../crates/engine-native/tests/groups/l_rounds.rs) 接入 `mod q4_remaining_fixes;`，新增 `26` 个 Q4 回归测试（`boss_play` 4、`boss_play_round2` 3、`boss_play_round3` 7、`boss_play_round4` 4、`boss_play_extra` 2、`martial_coin_fool` 3、`shousha_douzhuan` 2、`cardpile` 1）。

## 2. 台账变更（`Batch UI-7` 去重后净升 `+97`）

- 核对 `#114` 批准的 `104` 项及 `18` 个 UI 键后，确认其中 `7` 项及全部 UI 键已由 `Batch UI-7`（`2026-11-21`）预关闭；本次在 [inventory_status.json](../../../migration/g0/inventory_status.json) 精确升级剩余 `97` 项 `简化近似` 键为 `已复核等价`：
  - `L03-boss` `8 -> 8/0/0`、`L04-cardpile` `21 -> 21/0/0`、`L15-手杀全扩` `11 -> 14/0/0`、`L17-斗转星移` `1 -> 11/0/0`、`L21-武皇争霸` `7 -> 9/0/0`、`L26-笨蛋插件` `1 -> 6/1/0`、`U02-boss` `16 -> 16/1/0`、`U03-cardpile` `30 -> 30/0/0`、`U04-coin` `2 -> 7/0/0`。
  - 保留 `L26:configs:Fool_liegong` 与 `U02:modes:boss_play_arenaReady` 为 `简化近似`。
- 更新后全局计数：`4,004` `已复核等价` / `420` `简化近似` / `0` `缺失`（37 来源 `缺失=0`），同步更新 [native-remaining-ledger.md](../../../architecture/native-remaining-ledger.md)、[upstream-extension-reconstruction.md](../../../migration/upstream-extension-reconstruction.md)、[g0/README.md](../../../migration/g0/README.md) 及 9 份来源文档。

## 3. 原始浏览器 / CDP 实证哈希

目录 `docs/operations/evidence-raw/review-q4-remaining/` 下原始产物 SHA-256（未入库目录，仅登记路径与校验摘要）：
- `docs/operations/evidence-raw/review-q4-remaining/probe-q4-remaining.mjs`：`e84816890439a7d32bd6d91b18f50d48f6cf8cb32df7029e7f0a935de9f52160`
- `docs/operations/evidence-raw/review-q4-remaining/upstream-q4-remaining-cdp-results.json`：`dad782e0a96a830804005a5607cbf45a7b0d4c36481c0d6d48b3ea417b0bc86d`
- `docs/operations/evidence-raw/review-q4-remaining/verifier-q4-probe.mjs`：`da2605ca8ee76e24bc8906de8db6cabdc712c6171dacd87fa65d150ad786d7ce`
- `docs/operations/evidence-raw/review-q4-remaining/verifier-q4-results.json`（`pageExceptionCount: 0`、`blockedRequestsCount: 0`；含 `V1_jiwu`、`V2_baonu`、`V3_jiange_targets`、`V4_cardpile_clamp`、`V5_liuli`）：`f2daa3751b4d5ad9b0b89757a59529ecaf481473676b86e6717c2ce8facc8c97`
- `docs/operations/evidence-raw/review-q4-remaining/verifier-q4-r3-probe.mjs`：`79cfa7606082c7576ea164c38e0eaef22ede785a57e5755ccc361d9e74e2e1ff`
- `docs/operations/evidence-raw/review-q4-remaining/verifier-q4-r3-results.json`（`pageExceptionCount: 0`、`blockedRequestsCount: 0`；含 `R3_retieji_judge`、`R3_boss_baonuwash_after_hp_transform`、`R3_liuli_jiwu_and_retieji`）：`daab558581dc9eca27f1a542d88897231f25d441a4c084fea012e2378d2bca81`

## 4. 主线本地实测结果（`2026-11-24`）

- 门禁脚本：`check-source-lines.py`（`9889` 文件 `<= 2000` 行，Rust 非测试文件 `<= 1000` 有效行）、`check-native-layering.py`（`0` 向上引用）与 `recount_g0.py`（`来源数=37 缺失=0 []`）通过；`python3 -B -m unittest discover -s scripts -p 'test_*.py'` 共 `207` 项通过（`OK (skipped=1)`）。
- 严格编译（`RUSTFLAGS='-D warnings'`）：`engine-native-wasm --target wasm32-unknown-unknown` 与 5 个原生 crate `--all-targets` 零错误零警告。
- `5-crate`（`engine-native-kernel`、`engine-native-content`、`engine-native`、`engine-native-wasm`、`content-native-loader`，`cargo test --offline --no-fail-fast`）：`51` 个测试块（含 doc-test 共 `56` 块），`3257 passed; 0 failed; 6 ignored`（其中 `l_rounds` `318/0/0` 含 26 项 Q4 测试、`g1_shahai` `464/0/0`、`kernel` `436/0/0` + lib `7/0/0`、`swd` `135/0/0`、`g1` `395/0/1`、`e2e` `113/0/0`、`nc_rounds` `311/0/5`、`nc08` `142/0/0`、`nc08_upstream_a_k` `141/0/0`、`nc08_upstream_l_z` `209/0/0`、`engine-native` lib `15/0/0`、`engine-native-content` lib `5/0/0`、`engine-native-wasm` `27/0/0`、`content-native-loader` `12` 块 `48/0/0`〔lib `10/0/0` + 11 个集成文件 `38/0/0`〕）。
- `3-crate`（`engine-native-kernel`、`engine-native-content`、`engine-native`）：`15` 个测试块（含 doc-test 共 `18` 块），`3120 passed; 0 failed; 6 ignored`。
- `--workspace`：`152` 个测试块（含 doc-test 共 `178` 块），`8516 passed; 0 failed; 8 ignored`；`desktop-client`：`3` 块 `20 passed; 0 failed; 1 ignored`（`lib` `14/0/0` + `native_engine_host` `6/0/0` + `native_planner_measurement` `0/0/1` ignored）。
- Stage B 与跨端一致：`conformance` `19/19` golden（`ALL PASS`）+ `4/0/0` 单元测试；双 `wasm-pack` 产出一致，`H42 == W42`（`8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff`）、`H1 == W1`（`6fb9d95a59233864b01c4f00dbe345dcce7f5be36a96c2a0472fcf84c2c40eb8`）、`M42`（`manifest_version=1`、`manifest_hash=1121434187aa3d7083f1ee035f9d369c14e90d8e6a131e02c58a373a50fa94e0`、`chain=ok`）、`skill-vm content/standard/jianxiong.skill`（`ok=true`）；`check-native-wasm-cross-target.py` `standard` 40 步（`final_hash=6a59d996f801868eef0ddaa2ea08162f706811adcdea88a522780eac19edfcbd`、`native_wasm_match: true`）与 `huanle` 20 步（`final_hash=42d745661c22c4126cbc5bedb72e5d82b315b22ebae07d8ae3391b3dfbbe1e3d`、`native_wasm_match: true`）；`pnpm --dir apps/web-client` `typecheck` 通过、`test` `540/0/0`、`smoke:native` `83/0`、`build` `244 modules transformed`（构建目录已清理）。
- 并入记录：[ci-runs.md](../ci-runs.md) 登记 `U05-7d`（`8fa758996`）CI Run `45288232922` Attempt 1 `10/10 success`。
