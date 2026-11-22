# Batch U01-3（U01 3D精选 20 项收口→94/95；`dddanzhi` 冻结锁边界留待用户确认）独立复核与集成摘要

- **基线 HEAD**：`820aaddd64450d03c4cf66fdac808480872778ca`（Batch L11-3 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `45022962559` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-u01-3` `622387115..26412405d98234f2a176c7a9a7a9631c25d71323`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa` 保持不变
- **独立复核员**：`verifier-rules` 首轮（`#187` + Chrome CDP）→ `verifier-rules-2` 次轮独立复核（`#205` 验证全部 9 项修复后给出 **`APPROVE`**）
- **清单变更**：全仓 `2,027 / 2,397 / 0` -> **`2,047 已复核等价 / 2,377 简化近似 / 0 缺失 / 0 上游未定义`**（`U01 3D精选` 由 `74 / 21 / 0` 升为 **`94 / 1 / 0`**；`dddanzhi` 按 Section 3 Item 3① 维持 `简化近似`；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与架构行数拆分

1. **历史合并体与钩子协同**（与 `L01` / `L11` / `L25` 共存）：
   - `kernel/events/aux_history.rs`：因 `main`（Batch L11-1）已将 `events/vm_history.rs` 并入 `events/aux_history.rs`，将 `U01-3` 于 `vm_history.rs` 新增的 `u01_turn_trick_used_stamp` / `u01_zhuanshe_used` / `u01_cj_len` / `u01_cj_card` 记录完整迁入 `aux_history.rs::record_vm_history`。
   - `archive/known_prompts.rs`：于表尾依序保留 `L01` 与 `U01-3` 提示词追加，维护既定索引不变。
   - `content/host/filtering.rs`、`step/u08_batch_d_hooks.rs`、`d34_flags.rs::is_ducai3_forbidden`（合并 `is_dddlianer_forbidden`、`l25_gongxin_red_sealed` 与 `mdqianxi72`）、`kernel/events/turn.rs`、`kernel/equipment.rs`（`u01_equip_cancelled` + `l11_equip_from_hand`）与 `kernel.rs`（`commit_recover` + `on_hand_card_removed`）全部兼容合入。
2. **Rust 有效行数上限合规拆分**：
   - 将 `crates/engine-native-kernel/src/machine.rs` 中 `Ctx` 技能实例管理组（`insert_skill`、`ensure_skill`、`finish_removed_skill`、`remove_skill`、`skill_enabled`、`set_skill_enabled`、`add_skill_until_turn_end`、`expire_temporary_skills`、`reset_turn_limits`）按职责拆入 `crates/engine-native-kernel/src/machine/skills.rs`（`mod skills;`），使 `machine.rs` 低于 `1,000` 有效行上限且行为零变化。

---

## 2. 独立复核 20 项升级清单 ＋ 第 21 项保留边界（`#205`）

- **升格为 `已复核等价` (20 项)**（`skill.js`，`reviewer: "verifier-rules-2"`）：`dddxiaheng`、`dddfengzheng`、`dddyulv`、`dddyulv_old`、`dddfenye`、`dddshichao`、`dddyouxue`、`dddchengjing`、`dddduanbing`、`dddxiaoxing`、`dddfuyi`（字面忠实实现上游 `(!n)<2` 算子且经 #162 裁决叠加存活主公数）、`dddzhuanshe`、`dddweiqiu`、`dddlianer`、`dddshixing`、`ddddanggu`、`dddlanghuai`、`dddxuanlun`、`dddjiexing`、`dddbailei`。
- **按 Section 3 Item 3① 诚实维持 `简化近似` (1 项)**：`U01:skills:dddanzhi`（`u01_batch_d_asks.rs` 保留 `EventKind::PhaseBegin` / `Phase::Prepare` 触发而非上游 `global: "phaseBegin"`，否则破坏冻结归档基准 `u01-dddanzhi-pending-v107.ngsa`、`NGSA=107` 与 `RULES_VERSION=333`，同时保留既有共享引擎 `Taoyuan` 目标集与 `Duel` `card==0` 边界；21 项 `dddanzhi*` 回归全绿，是否重录归档待用户确认）。
- **回归测试入口**：`crates/engine-native/tests/u01_ddd_fixes_3.rs`（`common`、`part1_c`、`part2_d1`、`part3_d2`，通过 `g1` 运行，共 `21` 项真实命令流回归测试，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-u01-3/`）

- `b336b7574c6dd567e38632e159c589aef6fc64013b6bd1c47faf7ca03368223e` — `docs/operations/evidence-raw/review-u01-3/probe-u01-3.mjs`
- `70ece3df1166b601a7006359d8ad996969e0ed6a4d743ffbca268f94135dc08e` — `docs/operations/evidence-raw/review-u01-3/upstream-u01-3-cdp-results.json`（`blockedRequestsCount: 0`）

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`44` 组、**`2,840` passed / `0` failed / `6` ignored**（含 `g1` **`395/0/1`**、`l_rounds` **`235/0/0`**）
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py`（`native_wasm_match: true`）& `cross-target-hash.sh`：**`ALL CROSS-TARGET CHECKS PASSED`**
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,147` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client run test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`、`check-source-lines.py`、`check-native-layering.py`、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`、`recount_g0.py` 与 `preflight.py` 全过。
