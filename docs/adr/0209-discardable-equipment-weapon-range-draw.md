# ADR 0209：可弃置装备目标合同与武器范围摸牌

## 背景

旧 SIXIANG「望橹」允许技能拥有者在出牌阶段限一次选择场上一张自己可以弃置的装备牌。
装备失去后，若该实体牌是武器，其原装备者按武器攻击范围摸牌。旧实现把目标过滤、
`discardPlayerCard`、卡牌信息查询和异步顺序留在 JavaScript/UI 事件链，不能直接进入权威运行时。

## 决策

Rule VM ABI 136 增加：

- 主动目标合同 `has_discardable_equip`，在 `ActivateSkill` 提交时通过共享弃置保护查询验证每名
  目标至少有一张技能拥有者可弃置的装备；
- `discard_target_equipment_draw_weapon_range(player, target)`，由引擎向 `player` 提供
  `target` 当前合法装备的稳定快照，并强制精确选择一张；
- Choice 恢复时重新验证技能、观察者、参与者存活、原始选项、装备区归属和弃置保护；
- 先走普通装备弃置管线；若选择的是实体武器，再通过移动结算屏障令原装备者摸其
  `weapon_range` 张牌，异常的零范围按一张处理。

ABI 136 追加 HOST_EFFECT 148、continuation 100 和 `EffectRequest` 哈希标签 184。

## 结果

- 目标可包含技能拥有者，保持旧 `filterTarget` 语义。
- 受弃置保护的装备不会出现在选择中，客户端也不能伪造陈旧装备提交。
- 卡牌移动及其嵌套触发先于摸牌完成，旧 JavaScript 只保留为语义证据，不进入 npack、
  扩展目录或运行时。
