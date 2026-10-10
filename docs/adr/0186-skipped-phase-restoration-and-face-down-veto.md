# ADR 0186：跳过阶段恢复与翻至背面否决

## 背景

旧版 COLLAB「午马」由三个 JavaScript 子技能组成：成为其他角色锦囊目标后摸牌；在
`turnOverBefore` 取消正面翻至背面；在 `phaseAnySkipped` / `phaseAnyCancelled` 后直接再次调用
原阶段函数。新运行时不能执行旧 JavaScript，也不能让单个技能绕过统一阶段状态机。

## 决策

Rule VM ABI 111 增加三个通用声明：

- `owner_alive_targeted_by_other_trick`：只匹配活着的拥有者成为其他角色有效锦囊牌目标；
- `turn_face_down_cancel override 1`：在唯一 `turn_over` 原语提交前查询；只取消正面到背面，
  已经背面的角色仍可翻回正面；
- `restore_skipped_phase override 1`：所有判定、摸牌、出牌、弃牌跳过路径仍先写入公开
  `PhaseSkipped` 事件和跳过计数，再恢复执行原阶段。主动 `skip_current_phase` 记录事件后保持
  当前阶段，由既有阶段 continuation 完成剩余工作。

恢复是状态机策略而不是技能专用 Host effect，因此之后具有相同语义的内容只需声明修饰器。

## 版本与兼容

Rule VM ABI 从 110 提升到 111，Engine Semantics 从 157 提升到 158。Rule IR 保持 v76；
内容 schema、包格式、网络、回放和快照 schema 不变。`ModifierTarget` 与 `TriggerFilter`
变体只在末尾追加，没有新增 opcode、HOST_EFFECT 或 `EffectRequest`。旧 ABI 包继续由 ABI 111
主机加载。
