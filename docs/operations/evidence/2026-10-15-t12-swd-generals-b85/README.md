# T12 swd 武将面批 85：舒莞儿（活弹）

- 日期：2026-10-15
- 范围：一将 partial 转正——**76/79 名**（上批 75 将+1）；新增 HostBehavior tag 380
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（huodan 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd舒莞儿 (shu hp3, 女) | 活弹 huodan（陷阱 sxianjing defer） | SwdHuodan(tag 380)=ActivateSkill usable:1——SelectCard 红色手牌（消费段校验）→ TargetChoice 1-2 名任一存活（min1/max2）→ 自失 1 体力 + 1 名目标受 2 火伤 / 2 名各受 1 火伤（上游 contentBefore loseHp + selectTarget [1,2] 同口径） |

## 验证

- **定向回归** `swd_generals_b85_huodan`：单目标正例（红牌→p1 受 2 火伤 4→2 + 自失 1 4→3）；反例：无红牌 ActivateSkill 拒。
- **包装载**：74 将入册（cards.json 73→74）。
- **全量**：engine-native **797 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **76/79 名**（sxianjing 待字段批：存储陷阱牌+同花色响应对面）。
- 四项用户决策维持待用户裁决清单登记。
