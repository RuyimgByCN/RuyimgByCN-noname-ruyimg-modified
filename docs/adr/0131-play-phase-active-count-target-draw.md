# ADR 0131：出牌阶段主动技计数与类型化目标摸牌

## 决策

Rule VM ABI 41 增加三项通用声明能力：

```text
select_hand 1 1 card_sha
owner_is_target_active_used_at_least_N_this_play_phase
choose_target_draw(player, count, other|male_other|female_other|any)
```

每个技能实例保存当前出牌阶段的主动发动次数。进入新的出牌阶段时清零；离开出牌阶段时不立即
清零，使 `phase_play_end` 触发仍可读取刚结束阶段的计数。主动选牌合同的 `card_sha` 使用技能
拥有者视角下的有效逻辑牌名。目标摸牌效果先发布强制单目标 Choice，并在续体恢复时重新验证
选择者、目标存活状态、非自身限制和角色性别；双性角色同时匹配男性与女性目标域。

## 原因

旧版「燕语」通过扫描当前 `phaseUse` 的 `useSkill` 历史判断是否至少重铸两张【杀】。复用既有
每回合 `use_count` 会把同一回合内的额外出牌阶段错误合并，也会在离开出牌阶段后过早丢失信息。
通用的阶段作用域计数可继续支持类似“本阶段发动至少 N 次”的技能。类型化目标域则避免为男性、
女性或任意其他角色分别增加技能专用 Rust 分支，并让服务器统一防御伪造选择。

## 版本

- Rule VM ABI：41
- Rule IR：52
- Engine semantics：127
