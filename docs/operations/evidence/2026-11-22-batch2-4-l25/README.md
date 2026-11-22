# Batch Q2-L25（L25 祖安武将 终 18 项收口 → 257/257；Queue Item 2 全量收官）独立复核与集成摘要

- **基线 HEAD**：`89406fe50c5d33c349b4456b05a0b7724c439a5e`（Batch U01-2 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44995724474` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-q2-l25` `d9e1c9a1f..f36e736402f867df1cf9697b476935a1fe6abb79`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与冻结共存包 `u01-dddanzhi-pending-v107.ngsa`（`d3` `1_553_573_260_256_134_951`、`rest` `9_097_112_637_251_845_257`、`packs` `11_413_175_519_070_831_972`）保持不变
- **独立复核员**：`verifier-rules`（未参与 Batch Q2-L25 实现；结合 Round 1–2 `#149`/`#160` 复审与 Round 3 源码/状态机审计及 Chrome CDP 实跑探针，在 `#167` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,741 / 2,683 / 0` -> **`1,759 已复核等价 / 2,665 简化近似 / 0 缺失 / 0 上游未定义`**（`L25 祖安武将` 由 `239 / 18 / 0` 升为 **`257 / 0 / 0`**（100% 单项等价），**Queue Item 2（`L29`/`L10`/`L28`/`L27`/`L20`/`L22`/`L19`/`L25`）全量收口**；整包八项签署仍 **`0/37`**）

---

## 1. 集成冲突协调与并行修复

1. **`catalog/hooks.rs` / `content/hooks.rs` / `content/host.rs` / `resolution/{dispatch,frame_hooks}.rs`**：
   - 将 `ContentHooks::on_host_event` 加至拆分后的 `catalog/hooks.rs`（保留主线全部既有 `on_game_start`..`on_phase_skipped` 钩子）；在 `resolution/frame_hooks.rs::on_frame_popped` 的 `EventData::Damage` 清理后调用 `is_post_commit_host_event`→`(catalog.hooks.on_host_event)`，确保 `Damage`/`Recover`/`LoseHp`/`CardsDrawn`/`Death`/`TurnEnded` 仅在 `Stage::Done` 单次派发（不于 `BeforeTriggers` 重复触发）。
   - 在 `resolution/dispatch.rs::start_queued_trigger` 保留主线 `dddmoyan` (`DamageBefore` skip optional gate)，同步传入 `out` 至 `notify_new_skills` 以完成 `ssr_luoyi` 自动弃防具。
2. **`kernel/{combat_rules,death,duel,equipment,sha/{step,respond}}.rs` / `machine/{mod,helpers}.rs` / `play_{card,equip,sha}.rs`**：
   - `combat_rules.rs`、`sha/respond.rs::post_dodge_step`、`duel.rs` 与 `play_equip.rs`/`equipment.rs` 同时保留主线 `is_u01_hand_locked`、`u01_equip_from_table`、`dddqianlong`、`emit_duel_damage`（`d.card`＋`luoyi_bonus`），并叠合 `is_ssr_luoyi_trick_to_duel`、`needed_shan_count`（`ssr_luoyi` + `wushuang` + `liaoyuan`）、`guanfu` 止赏/`guanfu_disrecover` 与禁防具栏拒绝。
   - `l19_fixes.rs::host_preserves_skills_on_death` 按 `l25_zuan_mode` (2/4) 与 `SkillDef::trigger.event == Dying` 区分 `L19` 与 `L25` 同名 `_relifeskill`，避免 `L19` 轮始倒数误触发于 `L25` 复活局。
3. **`d34_flags.rs` 拆分与 `decision34_pack9_extension_pack()` 内容锁协同**：
   - 按 `AGENTS.md` 单文件 `<=1,000` Rust 非测试有效行上限，将 `apply_card_unrespondable` 拆入 `crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_unrespondable.rs`（`d34_flags.rs` 总计 978 行、有效行 `<= 1,000`，公开导出不变，历史行号引用零越界）。
   - `Q2-L25` 移除虚构牌 `ssr_jinfan` 并在 `decision34_pack9_extension_pack()`（空 `existing_packs`）加入 `pyzhuren_diamond` 与 `ansha_*`/`guanfu`/`guifu_*`（同时保留 `!existing_packs.is_empty()` 冻结 `u01-dddanzhi-pending-v107.ngsa` 锁 `9_097_112_637_251_845_257`）；同步更新 `crates/engine-native/tests/l20_l22_fixes.rs` 对应非冻结包锁期望至 `1_761_060_662_268_120_978`。

---

## 2. 独立复核 18 项升级实体清单（`#167` `APPROVE`，`祖安武将/extension.js`）

- `cards` (1)：`ssr_jinfan` (`:6535`，非实体牌；武将技 `discard_card`＋`ssr_jinfan` 弃牌堆＋距离/无视防具修饰器及 `onremove` 清理)
- `configs` (2)：`ssr_luoyi` (`:15031`，禁用/弃防具、普通/延时锦囊视为决斗、`Sha`+1 闪/决斗+1 杀)、`ssr_lieren` (`:22337`，首回合始向所有 `ssr_lieren` 持有者发 `pyzhuren_diamond` 武器及回合限 2 弃置加伤)
- `ui_hooks` (4)：`_anshaskill` (`:42659`)、`_seadreammodeskill` (`:42815`)、`_relifeskill` (`:43402`)、`_haoyunlai` (`:47584`)
- `skills` (11)：`_guanmodeskill` (`:42978` 含 `guanfu`/`guifu_*`)、`wumou` (`:48975`)、`new_guixin` (`:49077`)、`gongxin` (`:49506` 含 `l25_gongxin_temp`)、`_zuan_damageSendEgg` (`:40324`)、`_newZuanPhaseTX` (`:41709`)、`_zuanidentityshowskill` (`:42132`)、`_zuanCHANGEHPsayskill` (`:46389`)、`_fromsay` (`:47046`)、`_bingletexiao1` (`:48747`)、`_bingletexiao2` (`:48768`)
- 原生实现与测试：`crates/engine-native-content/src/content/host/step/l25_batch_a_modes.rs`、`crates/engine-native-kernel/src/kernel/l25_haoyunlai.rs`、`crates/engine-native-kernel/src/machine/helpers.rs`、`crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_l25_mods.rs` 与 `crates/engine-native/tests/l25_zuan_fixes.rs`（`10/10` 通过，`0` `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-q2-l25/`）

- `737a82457a03f6f9dbe5343a3c0867c8e54eb43200f5fbadcf0b9a1e955bc933` — `probe-q2-l25.mjs`
- `636f0f017c2ec328f13761455206f64293d5e336e37fab15b63d80566f5a8cb8` — `upstream-q2-l25-cdp-results.json`（`pageExceptionCount: 0`）

---

## 4. `main` 全量未过滤门禁实测记录（`CARGO_TARGET_DIR=/private/tmp/noname-target-integrator CARGO_INCREMENTAL=0`）

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-integrator-strict RUSTFLAGS="-D warnings"` native `--all-targets`（`engine-native-kernel`, `engine-native-content`, `engine-native`）与 `wasm32-unknown-unknown`（`engine-native-wasm`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`30` 组、**`2,743` passed / `0` failed / `6` ignored**（含 `u01_dddanzhi_regression` `20/20` 与 `l_rounds` `200/200`）
   - `conformance`：**`19/19` (`ALL PASS`)** & `3` 组 **`4` passed / `0` failed / `0` ignored**
   - `desktop-client`：`4` 组、**`20` passed / `0` failed / `1` ignored**
   - `check-native-wasm-cross-target.py` & `cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,091` passed / `0` failed / `8` ignored / `0` filtered**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`540` passed / `0` failed**
   - `build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：`test_check_source_lines.py`（`2 OK`）、`check-source-lines.py`（`9,520` 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`、`check_doc_line_refs.py --strict`（越界 `0`）、`recount_g0.py` 与 `preflight.py` 全过。
