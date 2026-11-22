# Batch U01-2（U01 3D精选 第二批 21 项技能收口 → 74/95）独立复核与集成摘要

- **基线 HEAD**：`9842e555d9fc336ffb5c37a09d0b815da1a0ae32`（Batch Spine-Tests 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44990992457` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-u01-2` `44c3ae911..6223871156befcb0276242a8810056a12c9c2d74`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules`（未参与 Batch U01-2 实现；经三轮源码/内核审计与 4 轮上游 headless Chrome CDP 探针比对，在 `#157` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,720 / 2,704 / 0` -> **`1,741 已复核等价 / 2,683 简化近似 / 0 缺失 / 0 上游未定义`**（`U01 3D精选` 由 `53 / 42 / 0` 升为 **`74 / 21 / 0`**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调（与 L11-1 / U01-1 / Q2-L20+L22 / Q2-L19 / Batch 1.6a / Q2-L27 共存）

1. **`catalog.rs` / `catalog/hooks.rs` / `content/hooks.rs` / `content/host.rs`**：保留主线 `catalog/hooks.rs` 拆分结构与全部既有钩子（`play_end_step`、`host_on_game_start` 等），在 `SkillBehavior` 保留 `runs_before_phase_begin`，在 `ContentHooks` 新增 `on_phase_skipped` 接入 `u01_phase_skipped_hook`，并在 `HostSkillBehavior` / `host_skill_limits` / `filtering.rs` / `u08_batch_d_hooks.rs` 接入 `u01_fixes_2`。
2. **`kernel/events.rs` 与 `kernel/events/aux_history.rs`**：不回退为临时 `vm_history.rs`，将 `U01-2` 在 `commit_recover_skills` 的 `turn_change_hp` 旗标合入 `aux_history.rs`，并在 `events.rs::commit_card_used` 接入 `play_phase_seq` / `card_use_play_{seq,idx}` 序号追踪。
3. **`resolution.rs` 与 `resolution/frame_hooks.rs`**：采用 `U01-2` 拆出的 `frame_hooks::on_frame_popped` 并在 `Damage` 弹栈清理中保留主线 `mbcmjiushi_dmg:{id}` 清理。
4. **`kernel/nullify.rs`、`kernel/dying.rs`、`snap/asks.rs`、`kernel/combat_rules.rs`、`sha/{step,respond,respond_skill}.rs` 与 `snap/asks/sha.rs`**：
   - `nullify.rs` 与 `snap/asks.rs` 同时保留 `host_response_skills(..., 3)`（`L10 reguhuo`/`scsanruo`）与 `(catalog.hooks.respond_skill_options)`（`U01`），并在 `blocks_all_cards` 门控下协同派发。
   - `combat_rules.rs`、`sha/respond.rs` 与 `snap/asks/sha.rs` 统一改用 `is_u01_hand_locked` 并保留 `has_rechanyuan` / `is_rechanyuan_sealed_name` / `target_sentinel`。
   - `d34_flags.rs` 同时导出 `d34_l11_mods`（`apply_draw_count`, `has_yaya_protected`, `is_card_ignored_in_discard_phase`, `txlianbi_blocks_unlink`, `txminguangkai_blocks_link`）与 `d34_u01_mods` 全量接口；`play_equip.rs` 同步保留 `l11_equip_use` 记录。

---

## 2. 独立复核 21 项升级技能清单（`#157` `APPROVE`，`skill.js:2005..3866`）

- `dddzhengjun` (`:2005`)、`dddxianxi` (`:2194`)、`dddlingyong` (`:2296`)、`dddxuxiao` (`:2333`)、`dddxuyu` (`:2407`)、`dddshijian` (`:2423`)、`dddxuyu_old` (`:2519`)、`dddshijian_old` (`:2574`)、`dddtaisi` (`:2636`)、`dddquche` (`:2706`)、`dddqianlong` (`:2819`)、`ddddongcha` (`:2851`)、`dddzhijie` (`:2941`)、`dddqiahua` (`:3068`)、`dddxunxun` (`:3112`)、`dddfusi` (`:3149`)、`dddfusi_global` (`:3174`)、`dddtuoji` (`:3369`)、`dddchashi` (`:3388`)、`dddmiaoxing` (`:3569`)、`dddfenji` (`:3866`)。
- 原生实现与测试：`crates/engine-native-content/src/content/host/step/u01_fixes_2/`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_u01_mods.rs`、`crates/engine-native-kernel/src/resolution/frame_hooks.rs`、`crates/engine-native/src/engine/play_equip.rs` 与 `crates/engine-native/tests/u01_ddd_fixes_2.rs`（29 项 `u01_fix2_*` 回归测试，全部经真实命令流且 0 `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-u01-2/`）

- `d31b9d17fa88122e4d55897335e13d1cf1492dd4f810bb886a88497dffc0dc3e` — `probe-u01-2.mjs` → `5881927813f0a9597680e0c63d3a98ff35c3915e1157db742ca39e5ee1f0d526` (`upstream-u01-2-cdp-results.json`)
- `4867b16f9a44d1ffe53b1f5163e56260cff0711a8ca72b3ffe08a3af2c6217ff` — `probe-u01-2-r2.mjs` → `324db1d710ae145399151826668ceeec5e268fa1765ca3a349ccc615fcb7b7ba` (`upstream-u01-2-r2-cdp-results.json`)
- `1eb41a8a9cca528bd6545b3fab9411d7f256ae155c0c0209d73557992b1420e8` — `probe-u01-2-r2b.mjs` → `f7fac64d42fd9d3e4374e5775c63111951a3955edec237da7d95e64592fe6dbc` (`upstream-u01-2-r2b-cdp-results.json`)
- `41307bc5c59d222f9f905c67a4ac116cc5ea2ac0cfdb822a331dbd07c24a075d` — `probe-u01-2-r3.mjs` → `3297f28d556fff7938f1200f443b97953d5b2d15b5974d91dd19145e6e93b88e` (`upstream-u01-2-r3-cdp-results.json`)

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（5 crate）与 `wasm32-unknown-unknown`（`engine-native-wasm`, `engine-native`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`30` 组、**`2,733` passed / `0` failed / `6` ignored**
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py` & `cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,081` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`（`2 OK`）、`check-source-lines.py`（9,515 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`（`10,922` 引用、越界 `0`）、`recount_g0.py` 与 `preflight.py` 全过。
