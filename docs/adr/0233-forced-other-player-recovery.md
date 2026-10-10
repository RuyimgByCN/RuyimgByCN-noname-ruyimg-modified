# ADR 0233：强制选择其他角色回复

## 决策

ABI 160 增加 `choose_other_recover(player, amount)`（HOST_EFFECT 169）。效果列出除选择者外
的全部存活角色，打开不可取消的单目标选择；恢复时重验目标仍存活且不是选择者，再复用普通
`RecoverHp` 管线。满体力目标仍是合法选项，恢复自然为空操作。

选择恢复复用 `ChooseOtherDrawOrRecover` continuation 的追加阶段，不新增 continuation、
包字段或角色专用状态。`EffectRequest::ChooseOtherRecover` 追加在枚举末尾，稳定哈希标签为
210；使用该效果的包要求 ABI 160。

## 原因

旧「献首」的目标选择是强制的，不能复用允许取消的
`choose_other_draw_or_recover(player, 0, amount, 1)`。追加一个通用效果可保留该差异，同时
复用现有选择协议和回复原语。
