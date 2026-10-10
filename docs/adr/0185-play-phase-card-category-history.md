# ADR 0185：主动技能的出牌阶段牌类别历史

## 背景

旧版 COLLAB「寅虎」把已弃置牌类别保存在临时子技能 storage 中，并在 `phaseUseAfter` 清理；
若本次伤害产生濒死事件，则另把主动技禁用到回合结束。类别历史与禁用标记属于不同生命周期，
不能合并为既有的整回合类别/目标历史，也不能在新运行时执行旧 JavaScript。

## 决策

Rule VM ABI 110 增加：

- `select_hand|select_he ... unused_category_by_skill_this_play_phase`：自动选择、直接提交和 Choice
  恢复都排除当前技能实例在本出牌阶段已记录的有效基本/锦囊/装备类别；
- `record_selected_play_phase_category(self_skill)`：要求精确选择一张牌，在弃牌前重验牌的归属、
  区域和类别，然后原子写入技能实例的 3-bit `play_phase_category_mask`。

每次真正进入 Play 阶段时，位图与既有 `play_phase_use_count` 一起清零；同回合内的
`active_turn_banned` 不清零。因此额外出牌阶段可以重新使用三种类别，但不能提前解除因濒死
产生的回合级禁用。类别记录不包含目标，允许用不同类别重复选择同一角色。

## 版本与兼容

Rule VM ABI 从 109 提升到 110，Engine Semantics 从 156 提升到 157，新增 HOST_EFFECT 128，
并在 `EffectRequest` 尾部追加公开变体。Rule IR 保持 v76；内容 schema、包格式、网络协议、
回放和快照 schema 不变。新增技能实例字段使用 serde 默认值且仅在非零时进入增量状态哈希；
旧快照保持可读，旧 ABI 包继续由 ABI 110 主机加载。
