+# ADR 0357：ABI 494 扬武红桃成本与差值取牌 + offline 樊稠

OFFLINE 樊稠（`ns_fanchou`，3362795240）仅有「扬武」（`nsyangwu`，
3480637807），以通用主动选牌约束 `card_heart` 和
`choose_gain_target_hand_ceil_half_difference(player, target)` 完成闭包。

## 决策

- 目标资格复用 `hand_gt_owner`，在主动技提交时、弃牌前校验。
- 引擎先弃置所选红桃手牌并等待移动及嵌套触发，再按目标与拥有者当前手牌差的
  一半向上取整；这与旧 JavaScript `content()` 在成本支付后计算一致。
- 拥有者私密选择准确数量的目标手牌，获得前公开这些牌；不保存技能专属状态。

## 验证

- `original_offline_nsyangwu.rs` 覆盖 ABI/武将闭包，以及“拥有者 2、目标 5 张手牌，
  弃置成本后差值为 4，获得 2 张”的时序。
- offline npack 与 `extensions/offline/` 两份发布包同步，catalog 中武将与技能标记
  `ported`；旧 JavaScript 仅作语义证据。

