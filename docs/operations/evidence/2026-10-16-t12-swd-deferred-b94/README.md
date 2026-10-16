# T12 swd deferred 消解批 94：破阵/鬼眼复活面

- 日期：2026-10-16
- 范围：两技 deferred 消解（tag 398-399）——**122/184 技条目已映射**（deferred 62→60）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（pozhen/guiyan2 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 破阵 pozhen（赵云） | 受伤后若与来源手牌数不同：多于时弃差值张对其 1 伤；少于时可弃其差值张 | SwdPozhen(tag 398)=Damage optional：确认→Act 读差值分流——多：SelectCards min=max=diff→弃→来源 1 伤；少：Confirm→弃来源首 diff 张（确定性登记） |
| 鬼眼·复活面 guiyan2（冯天凌） | 濒死时强制回 1 并失去鬼眼（group 子技随鬼眼使用而挂） | SwdGuiyan2(tag 399)=Dying forced priority 6：recover 1（**recover 自清 dying 旗→救援机循环顶自然 DyingSaved**）+移除鬼眼双技实例；鬼眼机器取牌成功段授予复活面实例（cards.json 顶层技能 `swd_guiyan2`） |

## 设计要点

- **guiyan2 授予/移除链**：鬼眼使用成功→`skills.entry((owner, swd_guiyan2 def)).or_insert`；濒死触发→recover+双 `skills.remove`（guiyan 本体+swd_guiyan2）——group 子技「随用随挂/一次性」完整闭环。
- **recover 清 dying**：`Ctx::recover` 在 hp>0 时自动 `dying=false`——濒死强制回复类技直接 recover 即可，救援机循环顶 `hp>0 && !dying` 自然发出 DyingSaved（归汉同口径）。
- 破阵少分支=弃**来源**的牌（chooseBool 确认后 trigger.source 弃置）——与多分支方向相反。

## 验证

- **定向回归** `swd_deferred_b94_pozhen_guiyan2`：破阵差 2 弃 2 张+来源反伤 1（Pass 出闪窗让伤害落地）；鬼眼取梅花+复活面实例授予→压血 1 致死量→强制回 1 存活+双技移除断言。
- **全量**：engine-native 811 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0（陈旧布阵变量清理后复跑绿）。

## 结论

- **122/184 映射，deferred 62→60**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
