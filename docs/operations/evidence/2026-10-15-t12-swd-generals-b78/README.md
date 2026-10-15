# T12 swd 武将面批㊳：姬亮

- 日期：2026-10-15
- 范围：一将 partial 转正——**64/79 名**（上批 61 将+1）；新增 HostBehavior tag 365
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（yunchou 逐段核验）

## 一将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd姬亮 (wu hp3, 男) | 运筹·姬亮面 yunchou（gongxin NOT FOUND/jqimou 回合外 chooseToUse defer） | SwdYunchou(tag 365)=ActivateSkill usable:1：SelectCard 弃一张手牌（色锚）→TargetChoice 有手牌目标→目标随机弃一张手牌：同色自己摸 1 否则目标摸 1 |

## 设计要点

- 色锚以 is_red 布尔入 card 槽（u32::from(bool)）——多段机槽复用惯例。
- 同色判定=双方花色 is_red 布尔相等（花色四分归并为红黑二色，上游 get.color 同口径）。

## 验证

- **定向回归** `swd_generals_b78_yunchou`：锚红+p1 全红手牌→随机弃必同色→自己摸 1（出 1 摸 1）+p1 弃 1；无手牌发动拒反例。
- **全量**：engine-native 790 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **64/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
