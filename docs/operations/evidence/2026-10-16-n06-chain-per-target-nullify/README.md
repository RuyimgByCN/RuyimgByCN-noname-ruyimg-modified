# 2026-10-16 N06：铁索连环无懈窗口逐目标对齐（规则 v290 / 存档 v77）

核验日期：2026-10-16。源码：`noname-next`，规则 `STANDARD_RULESET_VERSION = 290`、
存档 `ARCHIVE_FORMAT_VERSION = 77`。任务来源：[覆盖清单](../../../native-coverage-manifest.md)
行为差异表「铁索无懈仍为整牌级」（处置要求：按选定规则来源核对，再写独立行为断言，N06）。

## 1. 结论

覆盖清单该行为差异项**已关闭**：铁索连环的无懈窗口由**整牌级**改为**逐目标**——
抵消只作用于被抵消的那个目标，其余目标照常横置；整张牌仍只入弃牌堆一次。

## 2. 规则来源核对（选定来源：本地 `archive/upstream/noname`）

| 位置 | 事实 | 推论 |
|---|---|---|
| `apps/core/card/extra.js` `tiesuo` | `type: "trick"`、`complexTarget: true`、`selectTarget: [1, 2]`、`content: async (event) => { await event.target.link(); }`；**无** `multitarget: true` | 铁索是多目标但非 multitarget 的锦囊，效果逐目标执行 |
| `apps/core/card/standard.js` `_wuxie` | `trigger: { player: ["useCardToBegin", "phaseJudge"] }`；`map.target = trigger.target`、`map.tempnowuxie = trigger.targets && trigger.targets.length > 1 && !trigger.multitarget` | `useCardToBegin` **逐目标**触发 → 铁索 1–2 个目标各有一个独立无懈窗口；`tempnowuxie` 分支正是这类多目标非 multitarget 牌 |
| `apps/core/card/standard.js` `_wuxie.content` 末步 | `event.wuxieresult.useResult(...)`；无懈本体 `wuxie.content` 对目标事件调 `triggerEvent.neutralize()`，仅国战「对势力使用」分支追加 `excluded` | 抵消以**单目标事件**为单位；不扩展到整张牌 |
| `apps/core/card/standard.js` `_wuxie.filter` | `if (event.getParent().nowuxie) return false;` | 同一张牌的其余目标仍可继续开窗（只有整牌级禁令才阻断） |

结论：上游语义为**逐目标抵消**；原生原「整牌级窗口」是真差异，按对齐处理（而非仅登记）。

## 3. 实现变化

- `crates/engine-native/src/kernel.rs`
  - `ChainMachine` 由「整牌级单机器逐目标发 `Linked`」改为**派发层**：
    `card/source/targets/idx/inner/dispatcher`。派发层对**每个存活目标**生成
    `NullifyWindowMachine`（`owns_disposal=false`，队列 `seat_order_from(source)`），
    窗口内层是**单目标叶子** `ChainMachine`（`dispatcher=false`，未抵消时发
    `EventData::Linked` 触发横置翻转）。
  - `CardKind::Chain` 提交路径不再包 `wrap_nullify_window`（旧包裹在任一无懈生效
    时令全部目标都不横置）。
  - 收尾仍由派发层在所有目标结算后唯一弃置一次。
- `crates/engine-native/src/snap.rs`：`MachineSnapshot::Chain` 载荷
  `emitted` → `idx/inner/dispatcher`（`inner` 递归校验与重建）；
  `active_ask` 由 `None` 改为递归进入 `inner`（窗口挂起点恢复需要）。
- `crates/engine-native/src/archive.rs`：Chain 变体编码/解码同步（tag 9）；
  `ARCHIVE_FORMAT_VERSION` 76→**77**。
- `crates/engine-native/src/engine.rs`：canonical hash 的 `M::Chain` 分支同步
  （`idx` 入哈希、`inner` 递归、`dispatcher` 字节）。
- `crates/engine-native/src/content/standard.rs`：`STANDARD_RULESET_VERSION` 289→**290**。

## 4. 版本与 golden

- 规则 v289→**290**；存档 v76→**77**。
- 14 个 golden 场景重录（新哈希见 `replay_scenarios.rs` 表，日志
  `replay-golden-v290.log`）。
- **版本回退探针**（`probe-289-newcode.log`）：新代码 + 规则版本回退 290→289 +
  **golden 表回退为 v289 记录值** → **14 场全绿**。结论：新编码（`idx/inner/
  dispatcher`）不进入这 14 个哈希（这些场景没有挂起的 Chain 机器——铁索场景
  无人持无懈时窗口不挂起），位移唯一来源是规则版本字段。

## 5. 测试

新增/改写（`crates/engine-native/tests/native_kernel.rs`）：

| 测试 | 断言 |
|---|---|
| `chain_nullify_is_per_target_first_only` | 抵消目标 1 的窗口 → 1 不横置、2 **照常横置**；铁索只入弃牌堆一次；只有该窗口询问过 |
| `chain_nullify_is_per_target_second_only` | 目标 1 窗口放弃、目标 2 窗口出无懈 → 1 已横置、2 不横置（证明窗口不共享抵消状态） |
| `chain_double_nullify_leaves_nobody_linked` | 两目标窗口各被抵消（含同窗口无懈对无懈反转的放弃分支）→ 无人横置、铁索只弃一次 |
| `pause_roundtrip_chain_target_window`（`boundary_snapshot.rs`） | 铁索窗口挂起点保存→恢复→续答，逐点摘要/终态/事件增量与基准线一致 |

既有回归保持通过：`chain_card_toggles_link`、`fire_sha_propagates_through_chain`、
`thunder_damage_propagates_and_unlinks_all`、`chain_propagation_preserves_source`。

结果：

| 口径 | 结果 |
|---|---|
| engine-native 全量 | **806 通过、0 失败、1 ignore**（ignore 为既有诊断探针 `xly_state_growth_probe`）——[engine-native-tests.log](engine-native-tests.log) |
| Golden / 清单联动 | `cargo test --offline -p engine-native --test replay_scenarios`：2/2 通过（`replay_scenarios_match_golden` + `scenarios_are_registered_in_coverage_manifest`）——[replay-golden-v290.log](replay-golden-v290.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=/tmp/gate1b-target cargo check -p engine-native --all-targets`：**exit 0**——[gate1b-all-targets.log](gate1b-all-targets.log) |
| 版本回退探针 | 见上，14 场全绿——[probe-289-newcode.log](probe-289-newcode.log) |
| 全工作区 | 见 [workspace-tests.log](workspace-tests.log)（`cargo test --offline --workspace --no-fail-fast`） |

## 6. 登记边界（未随本批关闭）

- 同一窗口内的「无懈对无懈」仍按既有窗口语义反转抵消状态并自出无懈者下家重建
  队列——上游 `map.state` 经父 `_wuxie` 链同样翻转，语义一致，非差异。
- 铁索 `recastable`（重铸）面仍为既有未建模登记（本批不动）。
- 其余覆盖清单行为差异行（临时技能存续期、触发全序细节、濒死应答队列静态、
  死亡清技能、装备即时生效等）**不在本批范围**，仍开放。
