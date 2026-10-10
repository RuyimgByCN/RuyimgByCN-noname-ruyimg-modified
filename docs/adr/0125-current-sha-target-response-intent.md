# ADR 0125：当前【杀】逐目标响应意图

## 背景

旧「烈弓」在 `useCardToPlayered` 中向父事件的 `directHit` 数组追加当前目标。这不是角色的常驻
属性，而是一次实体【杀】、一个目标上的临时结算事实。把它建模成 `ShaTargetUnrespondable`
modifier 会跨牌泄漏；只保存一个整次用牌布尔值又会错误影响多目标【杀】。

## 决策

- Rule VM ABI 35 追加 `make_current_sha_target_unrespondable(source, card, target)`；host effect tag
  为 53，`EffectRequest` 与确定性哈希分别追加新变体和 tag 59。
- 效果仅允许出现在 `card_targeted` 触发中，并重新校验来源、实体牌和目标是否匹配当前保存的
  `ResumePlayedSha` / `ResumePlayedShaQueue` 续体。过期或伪造元组作为非法内容请求拒绝。
- 单目标续体复用原有响应布尔位。多目标续体在目标列表后增加 marker 与等长响应位段，取消目标
  仍保存在独立尾段；旧无 marker payload 继续按全部目标可响应解析。
- `owner_source_sha_target_hand_gte_hp_or_lte_attack_range` 过滤器读取统一攻击范围查询。武器、技能
  modifier 和模式全局加成不需要在「烈弓」中复制。

## 版本

- Rule VM ABI：35
- Rule IR：46
- Engine semantics：121
