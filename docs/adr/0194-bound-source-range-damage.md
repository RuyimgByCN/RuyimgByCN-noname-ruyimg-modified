# ADR 0194：绑定伤害来源的攻击范围目标选择

## 背景

旧版 `shenhua:quhu` 在出牌阶段限一次与一名体力值严格更高的角色拼点。未获胜时由拼点
目标伤害技能拥有者；严格获胜时，技能拥有者选择拼点目标攻击范围内的一名其他角色，再由拼点
目标造成伤害。旧 JavaScript 依赖 `event.target`、`_status.event.source` 和异步目标选择，
不能进入确定性内容包。

## 决策

复用既有主动选牌和双人拼点状态机，增加 Rule VM ABI 121 的两个最小通用语义面：

- 主动目标约束 `hp_gt_owner`，在激活提交时要求目标当前体力值严格大于技能拥有者；
- 效果 `choose_bound_source_attack_range_damage(player, source, amount)`，只接受当前技能
  实例保存的绑定目标作为伤害来源，由技能拥有者在该来源的当前攻击范围内强制选择目标。

拼点严格获胜时既有 `AddBoundSkill` 保存拼点目标，失败和平局仍进入同一未获胜助手。
Choice 打开和恢复时都重新验证技能实例、绑定来源、双方存活、目标存活及当前攻击范围；选择
目标可以是技能拥有者，但不能是伤害来源自己。一次性助手在结算后移除，不保存 JavaScript
闭包或事件对象。

## 结果

- `quhu`、`shenhua:xunyu` 与跨包 `mobile:re_xunyu` 均可作为完整 npack 内容发布。
- SHENHUA 提升至 ABI 121；MOBILE 自身仍只需要 ABI 120，通过既有精确依赖链取得 SHENHUA。
- Rule IR 提升至 83，Engine Semantics 提升至 166，追加 HOST_EFFECT 136 和 continuation 93。
