# ADR 0284：EXTRA 神陆逊与标记驱动目标约束

## 决定

旧 JavaScript 只作为语义合同，神陆逊的 `nzry_cuike`、`nzry_dinghuo` 重写为 DSL
和类型化效果，并与既有 `nzry_junlve` 闭合发布：

- 主动目标可声明 `linked`，命令入口权威校验目标当前已横置。
- `max_by_owner_mark <mark_def>` 令目标数上限取技能拥有者当前标记数，同时保留静态
  `targets` 上限。
- 摧克使用引擎状态机保存目标、弃牌选择和群体伤害进度；奇偶分支后的嵌套结算完成后，
  才判断是否提供清空八枚以上“军略”的选项。
- 绽火按座次依次弃置所选目标的全部装备；每次移动及嵌套触发结束后再继续，最后从仍存活
  的原目标中强制选择一名，造成有来源的火焰伤害。

## 兼容性

新增目标字段均有缺省值，HOST_EFFECT、`EffectRequest`、续体和稳定哈希标签只在尾部追加。
Rule VM ABI 为 208，Rule IR 为 94，Engine semantics revision 为 182；
内容 schema、包格式、网络、回放与快照 schema 不变。
