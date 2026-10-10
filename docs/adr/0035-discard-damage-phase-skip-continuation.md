# ADR 0035：弃牌/火焰伤害双分支与条件跳阶段

- 状态：Accepted
- 日期：2026-07-15
- Engine semantics：19

## 决策

新增参数化的 `choose_discard_or_fire_damage_skip_phases` 类型化效果。当前回合角色可取消，
或选择“弃置目标至多 N 张手牌/装备牌”“对目标造成 N 点火焰伤害”中的一项或两项；两项均选时
固定先弃牌、后伤害。目标、卡牌、分支位图和恢复阶段全部写入可序列化 continuation。

弃牌复用牌区移动与弃置保护查询，火焰伤害复用标准伤害管线，条件结果只设置既有
`SKIP_DRAW` / `SKIP_PLAY` 回合标志。`mbtanfeng.skill` 只声明触发和参数，旧 JavaScript
不进入包或运行时。

## 兼容性

效果 tag 26、continuation tag 22 和序列化枚举变体均追加在末尾，已有编号不变。该效果引入
新的回放可见结算语义，因此 `EngineSemanticsId` 从 18 升至 19；Native、浏览器 WASM 与
skill-vm 必须重新通过跨目标一致性门禁。
