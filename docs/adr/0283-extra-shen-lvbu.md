# ADR 0283：EXTRA 神吕布与回合临时技能

## 决定

旧 JavaScript 只作为语义合同，神吕布的 `wumou`、`ol_wuqian` 与 `ol_shenfen`
重写为 DSL 和类型化效果：

- 主动技能可声明 `mark_at_least <mark_def> <count>`，命令入口权威校验标记门槛。
- `add_skill_until_turn_end` 为新技能实例记录到期回合；已有同名永久技能不叠加也不被清理，
  后续永久授予会清除临时来源。
- `record_active_target` 复用主动技的回合目标历史，权威阻止同一技能重复选择同一目标。
- 神愤由引擎状态机保存其他存活角色的座次快照，逐人完成伤害、装备弃置和强制手牌选择，
  每一步均等待嵌套触发与濒死结算后再继续，最后翻面。

## 兼容性

新增字段均有缺省值，枚举、HOST_EFFECT、续体和稳定哈希标签只在尾部追加。
Rule VM ABI 为 207，Rule IR 为 93，Engine semantics revision 为 181；
内容 schema、包格式、网络、回放与快照 schema 不变。
