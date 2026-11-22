# Batch L11-1（L11 太虚幻境 32 卡牌＋首批 71 技能共 103 项规则收口）独立复核与集成摘要

- **基线 HEAD**：`a47f8b0e110cb217f54cb4f40f111a223dab649a`（Batch UI-7 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44981761423` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-l11-b1` `d9e1c9a1f..445ed1bd27beac8bf1e55273f896abc77214cb69`
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 静态内容锁保持不变
- **独立复核员**：`verifier-rules-2`（未参与 Batch L11-1 实现；经三轮源码核验与 10 项上游 headless Chrome CDP 实跑比对，在 `#135` 给出 **`APPROVE`**，`0` 剩余差异）
- **清单变更**：全仓 `1,617 / 2,807 / 0` -> **`1,720 已复核等价 / 2,704 简化近似 / 0 缺失 / 0 上游未定义`**（`L11 太虚幻境` 由 `6 / 930 / 0` 升为 **`109 / 827 / 0`**；整包八项签署维持 **`0/37`**）

---

## 1. 集成冲突协调与单文件行数拆分

1. **词汇表与宿主目录协同**（与 Q2-L27 / Batch 1.6a / U01-1 / Q2-L20+L22 共存）：
   - `vocabulary/host/{behavior,behavior_list,keys,metadata,tag_decode,tags}.rs` 与 `engine-native/src/lib.rs`：保留 `L27MbcmQingzheng(654)`、`L10ScsCompanion(574)`、`Rechanyuan(575)` 与全部既有映射，合入 `L11` 标签 `720..=727`（`HostBehavior::ALL: 580`）。
   - `crates/engine-native-kernel/src/catalog/hooks.rs` 与 `crates/engine-native-content/src/content/hooks.rs`：保留 `ContentHooks` 全部既有钩子（`host_play_begin_step` 等），新增 `play_end_step` 并接线 `phase.rs`→`play_end_rui_skill1`。
   - `content/host.rs`、`activation.rs`、`asks_mtg.rs`、`factory.rs`、`filtering.rs`、`step.rs`、`u08_batch_d_hooks.rs`（组合 `on_card_use_after_l11` + `clandianzhan` + `u01_card_use_after_hook`）、`kernel/equipment.rs`（保留 `dddzhilian`＋合入 `txhj_liulongcanjia` 双坐骑占格与 `record_l11_turn_card_used`）及 `kernel/events/aux_history.rs`（合入并替代临时 `vm_history.rs`，保留 `record_vm_history`/`record_dddanzhi_card`/`zhp_def`/`commit_recover_skills` 并加入 `record_l11_turn_card_used`）。
2. **单文件有效代码行数门禁拆分**（保持 Rust 非测试有效代码 `<= 1,000` 行及文档 `file:line` 引用零越界）：
   - `crates/engine-native-kernel/src/machine/helpers.rs`：从 `machine.rs` 拆出 `damage_data` / `damage_data_with` / `card_use` / `card_use_as` / `card_use_with_nature` / `note_cards_lost` 并经 `pub use helpers::*` 保持公开接口不变。
   - `crates/engine-native/src/ai/extension_choices_misc.rs`：从 `ai/extension_choices.rs` 拆出 `U08`/`L27` 挂起询问辅助策略 `plan_misc_choice_action`，保持 `L10`/`L28`/`U01`/`L11` 子模块路由与公开接口不变。
   - `crates/engine-native/src/engine/play_card_targets.rs`：接收从 `play_card.rs` 尾段拆出的 `push_play_card_frame`（含 view-as 与还侠跟踪），使 `play_card.rs` 低于 1,000 有效行的同时不改变既往文档引用行号。

---

## 2. 独立复核 103 项升级范围（`#135` `APPROVE`，12 项反例全闭环）

- **32 项 `L11:cards:*`**（`extension_card.js` 31 项 `txhj_xingtianpojunfu:8`..`txzhaoshu_global:2041` ＋ `extension_skill.js` `huogong:458`）：
  - 原生实现：`l11_cards.rs`、`l11_cards_asks.rs`、`l11_equips.rs`、`l11_equips_asks.rs`、`d34_l11_mods.rs`、`kernel/equipment.rs`、`play_card_targets.rs`（`chongzhu: true` 0 目标与 `_chongzhu` 重铸、`txhj_huoshaolianying` 单次无懈阻断整链并保留 `orig_card(m)` 伤害牌引用、`txhj_fangtianhuaji` 12>8 优先度与距离/空城/至少3人强配、`txhj_zhijizhibi` 空手零摸不泄漏、`txhj_chongyingshenfu` 卸装撤技能）。
- **首批 71 项 `L11:skills:*`**（`extension.js` `txhj_daiyuan:425`..`cxyPoLu:4331`）：
  - 侍从与代源 28 项（`txhj_daiyuan`..`txhj_qiaoqiaoSkill2`：`l11_servants.rs`、`l11_servants_asks.rs`、`kernel/nullify.rs`、`phase.rs`：`txhj_ruiSkill1` 于真实 `Phase::Play` 末判手牌数、被乐跳过不触发、双出牌阶段双触发；`txhj_tengsheSkill2` 限拥有者回合末且按存活敌方数择 2/1；`txhj_jiuweiSkill2` 按对手独立记戳；`txhj_qiaoqiaoSkill1` 按回合戳重装）。
  - 武将与太虚技能 43 项（`tx_mojian`..`cxyPoLu`：`l11_skills_{a,b,c}.rs`、`l11_skills_{a,b,c}_asks.rs`、`l11_common.rs`、`kernel/duel.rs`、`d34_l11_mods.rs`：`cxyFengYing` 仅护唯一最小体力友方、`txclanxieshu` 中文牌名长度与决斗保留 `d.card`、`txlieji` 免延时确认与 `tx_mojian`/`tx_yushou` 可选确认、`cxyKuangXi` 濒死即封印本回合）。
- **回归测试入口**：`crates/engine-native/tests/l11_taixu_b1_fixes.rs`（`cards`/`equips`/`servants`/`skills_a`/`skills_b`/`skills_c`/`round2_fixes` 共 `34` 项真实命令流集成测试，全部使用 `GameCommand`/`GameAction`，0 `debug_execute_effect`）。

---

## 3. Chrome CDP 上游原始证据索引（存 `docs/operations/evidence-raw/review-l11-1/`）

- `6d470a9bec16cf08c13c386fc13437f3c003632440b730056d41bf9299c64ae8` — `docs/operations/evidence-raw/review-l11-1/probe-l11-1.mjs`（10 项交互探针，`pageExceptionCount: 0`）
- `42ebef684096c6c35a18a3a5e3ea7eb9e2aac020703aadea6af80966fa3f425c` — `docs/operations/evidence-raw/review-l11-1/upstream-l11-1-cdp-results.json`

---

## 4. `main` 全量未过滤门禁实测记录

1. **Rust 编译与单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-l11-1 CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets`：exit `0`（已由 `EXIT` trap 清理 `/private/tmp/noname-target-l11-1`）
   - `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm -p engine-native`（`-D warnings`）：exit `0`
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native -p content-native-loader`：30 组测试输出，**`2,704` passed / `0` failed / `6` ignored**
   - `cargo run --offline -p conformance`：**19 / 19 (`ALL PASS`)**；`cargo test --offline -p conformance`：`4` passed / `0` failed / `0` ignored
   - `cargo test --offline -p desktop-client`：**`20` passed / `0` failed / `1` ignored**
   - `python3 -B scripts/check-native-wasm-cross-target.py`：`native_wasm_match: true`（标准 `40`、扩展 `20`）；`bash scripts/cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`
   - `cargo test --offline --workspace --no-fail-fast`：178 组测试输出，**`8,052` passed / `0` failed / `8` ignored**（exit `0`）
2. **前端验证**（`apps/web-client`）：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`；`pnpm --dir apps/web-client test`：**`527` passed / `0` failed**
   - `pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
3. **脚本与全仓门禁**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`（`2 OK`）、`check-source-lines.py`（9,496 文件 `<= 2,000` 行；Rust 非测试有效行 `<= 1,000`）、`check-native-layering.py`（`0`）、`check_ledger_consistency.py --strict`（`台账与枚举一致`）、`check_doc_line_refs.py --strict`（`10,903` 引用、越界 `0`）、`recount_g0.py` 与 `preflight.py` 全过。
