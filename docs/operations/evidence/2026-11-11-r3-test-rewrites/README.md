# R3 真实事件路径测试与测试边界规范登记

- **日期**：2026-11-11（2026-11-11 勘误与边界校准）
- **执笔 Agent**：`Worker_M2`
- **规则版本**：规则 333 / ABI 759 / NGSA 107
- **基准目标**：Milestone 2 - R2 逐来源规则等价重构 & R3 真实事件路径测试与测试边界诚实规范

---

## 1. 测试边界与遗留脚手架规范

为了准确界定真实事件驱动测试与遗留操作码回归测试的边界，杜绝夸大与虚假声明，明确规范如下：

1. **废弃遗留操作码直派接口**：
   - 内部测试辅助函数 `debug_execute_effect`（原名 `debug_commit_effect`）已标记为 `#[deprecated(note = "Legacy internal opcode harness; do not use in new R3 tests")]`。
   - 禁止在任何新的 R3 技能测试中使用该函数。
   - 既有 6 处历史回归测试套件（`decision34_round4_l11_taixu.rs`、`decision34_pack9_closure.rs`、`decision34_round4_l25_zuan.rs`、`decision34_round4_u05_shahai.rs`、`decision34_rules_and_campaigns.rs`、`up37_r02_to_n07_complete_closure.rs`）局部添加 `#[allow(deprecated)]`，作为底层操作码执行与状态转移的回归冒烟测试保留，但不作为 R3 规则等价性证明。

2. **真实事件驱动规范（Authentic Lifecycle Tests）**：
   - 仅限在 `crates/engine-native/tests/l11_taixu_boss_events.rs` 中编写的测试被认定为正规 R3 真实事件路径测试。
   - 所有测试均通过标准 `GameAction`（如 `NextPhase`、`PlayCard`、`Answer`）推入引擎状态机派发，完整穿越 `EventKind`、`VmFilterPolicy` 与堆栈结算。
   - 每个技能测试均包含反例测试（条件不满足不触发）。
   - 每个技能测试均包含跨存档快照 `save()` / `restore()` 挂起恢复一致性校验。

3. **台账与等价性边界（2026-11-11 Step 0 诚实校准）**：
   - `l11_taixu_boss_events.rs` 验证了 `boss_tianyun`、`boss_lingfeng`、`boss_chuanyun`、`leiji`、`dddxiaxing` 的阶段推进触发、不满足条件反例与跨存档 `save()`/`restore()` 恢复。
   - **但真实阶段推进测试不等于完整上游规则等价**：经代码走查，`dddxiaxing`（自动取首张手牌置顶，缺 `chooseCard` 玩家选择）、`boss_chuanyun`（自动选目标且可选发动未向玩家提问）、`boss_lingfeng`（缺玩家选择，且未 `trigger.cancel()` 替换正常摸牌阶段）与 `boss_tianyun`（缺玩家选择与可选发动提问，把上游 AI 体力判断写成了强制规则）均未达到上游等价标准。
   - 因此在 `docs/migration/g0/inventory_status.json` 中，上述 4 项均诚实保持为 `简化近似`（`已复核等价: 0 / 简化近似: 4,424 / 缺失: 0`），待补齐真实 `open_choice` 玩家选择与阶段替换语义后再行复核。

---

## 2. 真实事件路径改写明细台账

| 序号 | 技能名 | 上游真实规则及文件位置 (`file:line`) | 当前真实事件生命周期测试覆盖 | 反例覆盖 | 跨存档恢复 | 剩余上游差异（G0 状态） |
|:---:|:---|:---|:---|:---|:---|:---|
| 1 | `boss_tianyun` | `太虚幻境/extension_skill.js:4085`: 结束阶段可选发动，失去 1 体力，选 1 目标造成随机 2~3 点火伤并弃置其装备 | `Phase::End` 阶段推进、随机伤害与弃装备结算 | 体力为 1 不触发反例 | `restore_locked_engine` 快照恢复通过 | `简化近似`：缺玩家选择与可选发动提问（将上游 AI 体力判断写成了强制规则） |
| 2 | `boss_lingfeng` | `太虚幻境/extension_skill.js:3843`: 摸牌阶段改为亮出牌堆顶 2 张牌并获得；若颜色不同，可选 1 名敌方失去 1 体力 | `Phase::Draw` 阶段推进、亮出牌颜色比对扣血 | 牌堆顶同色牌不扣血反例 | `restore_locked_engine` 快照恢复通过 | `简化近似`：缺玩家选择，且未替换摸牌阶段（仍在正常摸牌基础上额外结算） |
| 3 | `boss_chuanyun` | `太虚幻境/extension_skill.js:3806`: 结束阶段，可选对一名体力大于自己的其他角色造成 1 点伤害 | `Phase::End` 阶段推进、`target.hp > player.hp` 过滤 | 目标体力不大于自身时不触发反例 | `restore_locked_engine` 快照恢复通过 | `简化近似`：缺玩家选择（自动选目标），可选发动未向玩家提问 |
| 4 | `leiji` (神话) | `shenhua/skill.js:7205`: 使用或打出【闪】响应杀时，选角色判定黑桃造成 2 点雷电伤害 | `PlayCard(杀)` 触发响应 `Answer(闪)`、触发雷击选目标提问与黑桃 2 点雷伤 | 出非闪或放弃响应不触发反例 | `restore_locked_engine` 快照恢复通过 | 核心包 `shenhua` 既有实现 |
| 5 | `dddxiaxing` (3D精选) | `3D精选/character/skill.js:59`: 准备阶段，可将 1 张手牌置顶并摸 1 张牌，本回合可如桃/酒使用 | `Phase::Prepare` 阶段推进、置顶与转化标记 | 手牌和装备均为空时不可发动反例 | `restore_with_extensions` 快照恢复通过 | `简化近似`：缺玩家选择（自动取首张手牌置顶），未向玩家发起 `chooseCard` |

---

## 3. 测试套件门禁通过证明

- **测试组注册校验**：
  `cargo test -p engine-native --test test_registry` -> `test every_test_file_is_registered_in_exactly_one_group ... ok`
- **真实事件套件校验**：
  `cargo test -p engine-native --test l_rounds -- l11_taixu_boss_events` -> `test result: ok. 5 passed; 0 failed`
- **全量 L 轮次校验**：
  `cargo test -p engine-native --test l_rounds` -> `test result: ok. 62 passed; 0 failed`
- **编译告警门禁**：
  `RUSTFLAGS="-D warnings" cargo check -p engine-native --tests` -> 0 告警通过（所有遗留冒烟测试均已显式标注 `#[allow(deprecated)]` 并与真实测试清晰隔离）。
