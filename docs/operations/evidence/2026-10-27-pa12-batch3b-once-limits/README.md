# PA12 迁移批③b：限次系语义验收 + 修复 AtPhase 元数据吞噬缺陷（2026-10-27）

- 执行时间：2026-10-27 **11:45:57 → 12:10 +08:00**（`HEAD = 23e06475`；裁决问询未获答复，按小步代理判断开工——批③勘正的收尾子批，证据与台帐注明）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §2 D 剩余项（once/round 限次系语义验收）

**一句话结论**：限次窗口两线对齐验收通过，并抓到/修复**第五个桥接缺陷**——批② 的 `AtPhase` 阶段门在机器工厂 `start()` 里让 `match &self.filter` 落入 `_ =>` 分支，**内层 Named 程序的 `consume`/`count_hits` 元数据被吞**：`phase_begin owner_alive_once_per_round`（AtPhase(Named)）的 once 消费不落 `vm_aux.once`，一轮内每个回合开始都重复开闸（实测一轮 3 门；裸 Named 的 liangfan2 正常）。修法：`named_meta` 递归透传。验收（真实技能全链 2/2）：良范（`damage owner_is_target_once optional`）同回合第二次受伤不再问、`vm_aux.once` 记录 (座位,键,scope1)、换回合清空再可触发；反转（`phase_begin owner_alive_once_per_round optional`）一轮只问一次（首门在 Judge 帧）、换轮再问一次。

## 1 窗口语义对齐依据（代码级）

| 面 | 旧线（engine-core） | 原生（engine-native） | 结论 |
|---|---|---|---|
| 每回合一次 | `turn_fire_count` 每回合开始**对全部技能实例**清零（command/turn.rs:88 遍历 sids） | `commit_turn_ended → vm_aux.clear_turn()`（scope 1 全座位 retain 清除） | 一致：任一回合内一次 |
| 每轮一次 | 滤名装载期设 once_per_round 限次 | `round += 1 时 clear_round()`（scope 2）+ `Seed::Once(2)`（`once_per_round` 短语，`take_known` 最长匹配实测 Once{2}） | 一致 |
| 消费时机 | 触发结算计数（含拒确认？未验） | VM **Halt 后**写 `vm_aux.once`（decline/取消不消费） | 差异登记：取消是否消费两线均未验收（后续子批） |
| 首回合帧 | TurnBegan 首回合也发 | construction 吞首回合 Judge begin（批②登记） | 已登记差异 |

## 2 验收（真实 ported .skill 全链）

| 测试（`pa12_batch3b_once_limits.rs` 2/2） | 断言 |
|---|---|
| 良范（offline，`damage owner_is_target_once optional` 摸 1） | 同回合首次受伤：门 + 摸 1 + `vm_aux.once` 含 (1,键,1)；第二次：无门不摸；换回合 scope1 清空、再受伤再触发 |
| 反转（collab，`phase_begin owner_alive_once_per_round optional`） | 一轮内恰一 门（Judge 帧，hp 2→3 回 1）；下一轮再恰一门（修复前一轮 3 门——AtPhase 吞 consume） |

## 3 变更与回归

- `rvm_bridge.rs`：`start()` 的 `named_meta` 递归提取（AtPhase→inner Named 的 consume/count_hits）。
- 回归（[b3b 汇总](../2026-10-26-pa12-dsl-native-prototype/b3b-regressions.txt)，12:05–12:09）：engine-native **1576/0/71**（+2）、loader 32/0、wasm 52/0、game-server 358/0、engine-core 3841/0/1。
- 其余门禁（workspace check、前端、行数/链接/ADR）见提交信息。

## 4 遗留

- 取消（拒确认）时 once 是否消费：两线均未验收，登记后续子批。
- `twice_per_turn`/`AtLeast` 系、`once_per_game`（scope 3）未逐一驱动（机制同一 `vm_aux.once`）。
