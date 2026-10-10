# ADR 0383: ABI 514 兵势庞羲（蓄业/狂想）

- 状态：已完成
- 日期：2026-08-02
- ABI：514

旧 JavaScript 只作为规则语义证据。新运行时以 DSL 声明技能，并用两个窄原语复用现有摸牌、移牌、选择和技能状态能力：`resolve_mbxuye` 处理伤害后摸牌及条件置顶，`resolve_mbkuangxiang` 处理整手交换、交换牌耗尽追踪和派生蓄业。主动目标新增对称约束 `hand_lte_owner`，由编译器和引擎统一校验。

门禁：庞羲 Golden 3/3、兵势包闭包、移植角色闭包、content-compiler 462/462、Rule VM、content-runtime、扩展导入与 conformance 8/8 全部通过。
