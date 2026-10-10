# ADR 0232：移除角色全部当前技能

## 决策

ABI 159 增加 `remove_all_skills(player)`（HOST_EFFECT 168）。引擎对目标当前技能实例生成按
实例 ID 排序的快照，再逐项复用既有 `remove_skill`，同步清理玩家技能表、触发器注册和修正器
注册，并逐项发出 `SkillRemoved`。

## 理由

旧 JavaScript 的 `clearSkills()` 表达通用规则效果。复用既有单技能移除路径，可避免为「断肠」
复制一套技能表和注册表清理逻辑；快照也避免遍历时修改集合。

## 兼容性

包格式、continuation 和索引结构不变；使用该效果的包要求 ABI 159，并新增
`EffectRequest::RemoveAllSkills` 及稳定哈希标签 209。
