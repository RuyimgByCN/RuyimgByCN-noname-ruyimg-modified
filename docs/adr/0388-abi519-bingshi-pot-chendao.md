# ADR 0388: ABI 519 兵势陈到（往烈/弘毅）

- 状态：已完成
- 日期：2026-08-02
- ABI：519

旧 JavaScript 仅作为语义证据。`resolve_potwanglie` 在出牌阶段开始选择一张实体手牌，并复用技能实例的精确牌策略，使该牌无距离限制且不可响应；该牌结算结束后清除精确策略并添加阶段临时限制，令拥有者本阶段后续用牌只能指定自己，出牌阶段结束时统一清理。

`resolve_pothongyi` 复用技能定义号作为“毅”标记号：游戏开始获得四枚，每次造成或受到伤害后增加一枚且上限五枚；准备阶段捕获当时的 X 并强制选择摸 X 或清空标记，结束阶段执行另一项。状态只保存在弘毅技能实例中，不引入 JavaScript 运行时或通用的新状态框架。

门禁：势陈到 Golden 3/3、兵势全量 Golden、content-compiler 467/467、Rule VM、content-runtime、engine-api ABI 协调、26 个扩展包导入与 conformance 8/8 全部通过；既有 conformance 哈希未漂移，无需重钉。全量 engine-core tests 的编译另被 `original_xianding_batch4.rs` 引用不存在的旧 `ContinuationKind` 变体阻断，与本 ADR 无关。
