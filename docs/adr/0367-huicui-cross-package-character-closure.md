# ADR 0367：HUICUI 孟优、界阚泽跨包技能闭包

此前 HUICUI 包中的孟优与界阚泽只装配了本包新技能，遗漏旧 JavaScript 角色定义中的
既有跨包技能，导致“完整武将”声明与实际包不一致。

## 决策

- `mengyou` 装配 XIANDING 包的 `manyi` 与 HUICUI 的 `dcmanzhi`。
- `re_kanze` 装配 SP 包的 `xiashu` 与 HUICUI 的 `rekuanshi`。
- HUICUI 清单显式声明 `xianding`、`sp` 依赖；不复制技能，不新增 ABI。

## 验证

黄金测试校验两个角色的技能顺序和依赖清单；全量角色闭包测试负责确认依赖包提供对应
技能定义。
