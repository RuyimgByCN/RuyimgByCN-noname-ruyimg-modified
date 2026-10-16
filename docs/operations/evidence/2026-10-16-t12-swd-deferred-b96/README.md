# T12 swd deferred 消解批 96：唤魂/倾城

- 日期：2026-10-16
- 范围：两技 deferred 消解（tag 402-403）——**126/184 技条目已映射**（deferred 58→56）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（huanhun/lqingcheng 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 唤魂 huanhun（小雪） | 任意角色濒死时弃一张牌令其判定：红→回 1；黑→其获得弃置牌 | SwdHuanhun(tag 402)=Dying optional priority 6：确认→弃 he 一张→内联判定→红 recover/黑弃置牌入手濒死者（弃堆守卫：中间被取走则不获得，上游 isInPile 同口径） |
| 倾城 lqingcheng（莉莲） | 结束阶段判定循环至多 3 次，红判定牌收入手牌，非红弃置终止 | SwdLqingcheng(tag 403)=PhaseEnd frequent：确定性顺序判定（内联口径）——红入手+继续、黑弃置+break；含洗弃堆防御 |

## 验证

- **定向回归** `swd_deferred_b96_huanhun_lqingcheng`：唤魂濒死窗（含确认门/救援窗多窗排队应答）流程收束；倾城压顶红红黑→两红入手+黑弃置终止+净增 2。
- **全量**：engine-native 813 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 测试布阵勘误（入记忆）

- **触发 filter 的持有者资源门**：huanhun filter 要求 owner he 非空——测试授予技后必须给持有者布资源牌（首版无窗实证：filter 静默拒绝不 panic）。

## 结论

- **126/184 映射，deferred 58→56**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
