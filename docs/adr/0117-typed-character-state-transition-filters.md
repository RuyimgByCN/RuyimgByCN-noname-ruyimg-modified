# ADR 0117：类型化武将牌状态方向过滤器

## 背景

统一的 `character_state_changed` 已覆盖翻面和横置布尔状态变化，但最初只暴露“发生变化”这一
事实。旧法恩、界法恩在翻至正面或背面后都可发动，却只在角色进入横置状态时发动；新杀法恩
和一将成名法恩则进一步只接受翻至正面。若只使用无方向过滤器，会让解除横置或错误的翻面方向
多触发一次。

## 决策

`TriggerContext.amount` 对 `character_state_changed` 使用稳定方向标签：翻至正面、翻至背面、
变为横置、解除横置。新增两个通用过滤器：

- `owner_alive_event_player_turn_changed_or_became_linked`：接受两个翻面方向和变为横置；
- `owner_alive_event_player_became_face_up_or_linked`：只接受翻至正面和变为横置。

两个过滤器都要求技能拥有者仍存活，且都排除解除横置。`nsqiyue` 继续使用无方向
`owner_alive`，所以仍接受全部真实状态变化。旧 JavaScript 的 AI 检查和全局提示助手不进入
权威运行时。

OLD、REFRESH 与 YIJIANG 包分别声明对应过滤器并生成 npack；四个法恩技能共享实现能力但保留
各自稳定定义 ID。

## 版本

该能力要求 Rule VM ABI 27、Rule IR 38、引擎语义修订 113。ABI 27 主机继续按加法兼容规则
加载 ABI 3–26 内容包。
