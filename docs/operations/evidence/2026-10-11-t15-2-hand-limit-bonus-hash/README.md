# 2026-10-11 T15②：旧线 canonical hash 补编码 `hand_limit_bonus`

任务来源：[进度主稿 T15②](../../../architecture/native-progress.md)；开工 HEAD `8bd3ec9a`。缺口此前登记于 [Arc 化记录](../../../migration/content-registry-arc-split.md)（"不在 canonical hash 编码内——旧线哈希覆盖待核对项"）。

## 决策：补编码（非有意排除）

`hand_limit_bonus`（`GameState` 上的 `BTreeMap<u32, i32>`，按玩家的手牌上限修正）是**真实玩法面**：

- 写入点：`command/resolve_choice/group_post_3.rs`（Muyin 选择结算，令目标本回合手牌上限 +1）；
- 读取点：`query/mod.rs`（弃牌阶段手牌上限查询的响应值）。

两个仅此字段不同的状态在旧 hash 下哈希相同——事务一致性对照、回放比对无法发现这类分歧，属于一致性缺口而非"可安全排除的派生面"，故排除方案不成立。

## 改动内容

`crates/engine-core/src/hash.rs` `canonical_state_hash`：按既有**条件编码惯例**（同 `turn.hand_limit_exempt_cards` 等）新增：

- 空表不写任何字节 → **既有哈希全部不变**（旧线 golden 与既有一致性测试兼容，无需按第 4 条重录——它们从未覆盖该字段，这正是缺口本身）；
- 非空时写 `b"state-hand-limit-bonus/v1"` + 长度 + 逐条 `(player: u32, bonus: i32)`——用到 Muyin 类技能的流哈希开始正确区分该字段。

回归测试 `hash::tests::hand_limit_bonus_enters_canonical_hash`：非空改变哈希、不同玩家不同哈希、同玩家不同值不同哈希、空表与 default 哈希一致（条件编码兼容性）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-core --lib hash` | **12/12**（含新回归） |
| `cargo test --offline -p engine-core --no-fail-fast` | **3778 通过 / 0 失败 / 1 ignore**（3777+1，exit 0）——既有 golden 与一致性测试零失败，证明条件编码完全向后兼容，无 golden 重录 |

## 边界

- Arc 化记录同表登记的**其余** journal 缺口（`group_overrides`、judge 区、turn 旗标、zones 结构性变化等不在 canonical hash 或不经 journal 的面）不在本项范围，维持原登记；`hand_limit_bonus` 在该记录中的"待核对"状态已由本批关闭（该页为带日期快照，按惯例只加注不改写）。
- 原生线（engine-native）的哈希不涉此项。
