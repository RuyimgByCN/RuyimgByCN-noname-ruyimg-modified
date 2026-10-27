# PA12 迁移批①：桥接词汇对齐（owner_alive / owner_is_source 别名 / CardUsed 事件来源 / 推迟滤镜装载期拒绝）（2026-10-27）

- 执行时间：2026-10-27 **00:20:00 → 09:33:00 +08:00**（实际 `date` 输出；其间宿主休眠约 9 小时——五个全量回归实际于 09:29:04–09:32:37 执行，见 [b1 汇总](../2026-10-26-pa12-dsl-native-prototype/b1-regressions.txt)；`HEAD = 26889e89`，PA12④ 原型批之后；用户已于 2026-10-26 23:5x 确认开批①）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) 批①；[PA12④ 原型证据](../2026-10-26-pa12-dsl-native-prototype/README.md) 为前批

**一句话结论**：四个桥接改动 + 三个真实技能端到端验收——① `VmFilterPolicy::OwnerAlive`（DSL `owner_alive`，754 处）；② `map_vm_filter` 别名 `owner_is_source`（206 处，与既有 `owner_source` 同义）；③ `trigger_ctx_of` 补 `CardUsed` 的 `event_source = 用牌者`（旧线 TriggerContext.source 语义；此前 `owner_is_source` 对用牌事件恒假）；④ 装载器对不承载滤名从「静默跳过」改为 **`BridgeError::UnsupportedVmFilters` 装载期拒装并列名**（`VmBridgeOutcome` 移除 `deferred_filters` 字段）。盘点：滤镜命中 1,463→**2,403**、推迟 2,358→**1,418**，文件级近似可直接原生 579→**1,054**（offline 92→310、huicui 69→98、shahai 0→17）。

## 1 验收（真实 ported .skill，正式装载路径）

| 测试（engine-native `tests/pa12_batch1_vocabulary.rs` 3/3） | 断言 |
|---|---|
| `pa12_batch1_vocabulary_mapping_and_strict_reject` | 映射面（别名/新策略）；未知滤名 `pa12_not_a_native_filter` → `UnsupportedVmFilters{filters:[...]}` 拒装 |
| `pa12_batch1_owner_alive_triggers_on_others_end_phase`（清励，clan 包） | 0 号（非持有者）结束阶段触发（`owner_alive` 不含事件主玩家合取），持有者（座位 1，手牌清空）摸 5——**两线一致的「再查询陈旧」**：见下 |
| `pa12_batch1_owner_is_source_after_own_card_used`（悟道，collab 包） | 他人用牌不触发；持有者自己出杀 → optional 确认门（trigger_confirm）→ 确认后摸 1（=打出的杀补回）——验证 CardUsed 事件来源补齐 |
| engine-core `pa12_dsl_legacy_differential.rs::pa12_legacy_clanqingli_requery_is_stale_draws_five`（新增 3/3） | 旧线同一模块同为 5 张：两线 VM 宿主都在模块 Halt 后落账，模块内 `hand_count(self)` 再查询看执行前状态，`loop 5 { if hand<max … }` 恒摸 5 |

**登记（迁移计划 §2 新 H 类）**：清励上游语义「摸至体力上限（至多 5）」在两条 VM 宿主线上都不可由「再查询循环」表达（效果排队、查询陈旧）——属 ported DSL 保真度问题，两线一致，非原生桥接缺口；需要「循环内即时落账」宿主原语或改写 DSL（先算张数再一次性摸）。

## 2 变更与回归

| 文件 | 变更 |
|---|---|
| `crates/engine-native/src/rvm_bridge.rs` | `OwnerAlive` 策略 + 求值；`owner_alive`/`owner_is_source` 映射；`CardUsed` ctx 补 `event_source` |
| `crates/content-native-loader/src/lib.rs` | `UnsupportedVmFilters` 错误 + 拒装路径；`VmBridgeOutcome{decls}`（移除 deferred_filters，补 `#[derive(Debug)]`） |
| `crates/engine-native/tests/t8_rvm_bridge.rs` | `t8_vm_skill_loader_rejects_unsupported_filters`（原 skips 测试改语义）；移除 4 处 deferred 断言 |

回归（[b1 汇总](../2026-10-26-pa12-dsl-native-prototype/b1-regressions.txt)，2026-10-27 09:29:04–09:32:37）：

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | **1567 / 0 / 71**（+3 本批测试） |
| `cargo test --offline -p content-native-loader` / `-p engine-native-wasm` / `-p game-server` | **32/0/0、52/0/0、358/0/0** |
| `cargo test --offline -p engine-core --no-fail-fast` | **3839 / 0 / 1**（+1 差分测试） |
| `cargo check --offline --workspace --all-targets` / 前端 test+typecheck / 三门禁 | 见提交信息 |
| 盘点重跑 | 滤镜命中 2,403、推迟 1,418、近似可直接原生 1,054（[批①后](../2026-10-26-pa12-dsl-native-prototype/inventory-after-batch1.txt)） |
