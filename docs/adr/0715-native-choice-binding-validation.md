# ADR 0715：选择承接关系逐字段校验（N04-e 关闭）——由机器状态推导待答询问

- 状态：已接受（2026-09-18）。
- 关联：`crates/engine-native/`（`src/snap.rs`、`src/engine.rs`）；
  前序：[ADR 0712](0712-native-first-official-skills.md)、
  [ADR 0714](0714-native-guidao-tiandu-semantics-n04d.md)。

## 独立复核（2026-09-20）

N04-e 原队首/observer 错配确实已拒绝，手牌主动改判与天妒确认/放弃的合法恢复探针通过。
但后续装备扩展只改了 kernel，active_ask 仍仅含黑色手牌：同时有黑色手牌/装备的正常询问保存后，
restore 报 OpenChoiceMismatch（N08-c）。因此“逐字段一致”是已实现检查，不等于所有合法机器状态已验收。
新增选项须同步实际询问与推导，并保留真实装备的正向恢复测试；具体反例见 [当前进度](../architecture/native-progress.md)。

## 背景

复核探针复现 **N04-e**：把改判队列首（2）改成 3、保留 2 号的开放选择与
手牌，`restore` 仍成功——恢复后 2 提交其合法黑牌，改判机器会对 3 执行
`take_hand_card(3, …)` 而 panic。根因：此前的选择承接校验只要求
「开放选择 ⇒ parks 非空」，未验证开放选择与续行机器**待答询问**的对应。

## 决策

### 1. 由机器状态推导待答询问（`active_ask`）

新增纯函数 `active_ask(machine, core, catalog) -> Option<ExpectedAsk>`：
从任一挂起机器（含嵌套子机器）推导其**当前应当开放的询问**——观察者、
提示、种类、选项（含顺序）、数量上下限、放弃许可。覆盖全部可询问机器：
杀响应、濒死救援、决斗、无懈窗口、AOE 响应、弃牌上限、可选触发确认、
鬼道改判、天妒获得、散财两段、移伤目标。瞬态（尚未询问、已消费、子事件
边界）返回 None——它们不会成为保存点上的开放选择。

### 2. 恢复时逐字段比对

`Engine::restore` 在既有校验后新增：
- 开放选择存在 ⇒ **恰好一个**挂起机器推导出询问，且观察者/提示/种类/
  选项（含顺序）/min/max/allow_cancel 与开放选择**逐字段一致**；
- 无开放选择 ⇒ 任何挂起机器都不得处于待询问状态。
不一致返回新错误 `OpenChoiceMismatch { observer }`。这同时覆盖 N04-d 类
（篡改 asked）与 N04-e 类（改判队首与观察者错位）的反例。

### 3. 测试

- `restore_rejects_rewrite_asked_with_empty_rewriters`（N04-d 回归，
  ADR 0714 批次）。
- `restore_rejects_sha_asked_tamper`：篡改 Sha 机器 `asked`（true→false）
  使开放选择失去承接 → `OpenChoiceMismatch`；机器字段的摘要敏感性由
  src 单元测试 `park_machine_state_changes_digest` 覆盖。
- `restore_rejects_choice_without_machine` 等既有负向测试继续通过；十个
  恢复矩阵案例在所测询问点验证推导与实际选择一致。鬼道只答 Pass、未授予天妒，
  不覆盖主动改判后的全部选择状态；装备恢复缺口见本次复核。

## 验证（2026-09-18，`cargo test --offline -p engine-native`，共 112 passed / 0 failed）

边界/快照 43 项（含上述新增与改写）、机制 64 项、场景 2 项（14 golden，
版本 5）、调试 1 项、单元 2 项全绿；无新增编译警告。

## 当批限制与后续（最新优先级见当前进度）

- 推导覆盖当前全部可询问机器；新增可询问机器时须同步实现其推导分支
  （与快照五处联动同列为新增机器的固定步骤）。
- N09（新线性能基线）为下一个主要批次。
