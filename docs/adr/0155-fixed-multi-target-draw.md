# ADR 0155：固定数量多目标摸牌

## 决策

Rule VM ABI 80 追加通用效果：

```text
choose_targets_draw(player, count, target_count, other|male_other|female_other|any)
```

`target_count` 是编译期校验的 `1..255` 常量。权威引擎仅在合法存活目标数不少于该值时打开
不可取消的选择，要求提交恰好该数量且互不重复的目标，并在续体恢复时重新校验存活状态和目标
作用域。实际摸牌顺序由当前存活座次决定，不采用客户端提交顺序。

## 兼容性

- 新 VM effect tag 为 `106`，新 `EffectRequest` 散列 tag 为 `135`。
- `StmtIr` 与 `EffectRequest` 变体均追加在枚举末尾，保留既有序列化序号。
- 既有 `choose_target_draw`、effect tag `59` 和四字段续体保持不变；同一续体解析器同时接受 ABI 80
  的五字段多目标载荷。
- 只有使用新效果的包要求 ABI 80；旧包继续按其原最低 ABI 加载。

## 首个使用者

四象标刘巴「锻币」先用既有 `discard_hand(self, 2147483647)` 支付全部手牌成本，再调用
`choose_targets_draw(self, 2, 2, any)`。目标可包含技能拥有者；若场上只剩一名存活角色，弃牌
仍结算但不会打开目标选择，保持旧 JavaScript 语义。
