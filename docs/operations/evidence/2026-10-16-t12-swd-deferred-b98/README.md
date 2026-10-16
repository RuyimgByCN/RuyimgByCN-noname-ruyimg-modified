# T12 swd deferred 消解批 98：拷贝栏/奥剑

- 日期：2026-10-16
- 范围：两技 deferred 消解（tag 406-407）——**131/184 技条目已映射**（deferred 53→51）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（kbolan/aojian 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 拷贝栏 kbolan（肯迪） | 每当摸牌时额外摸 1，摸牌结束将一张手牌置于牌堆顶 | Draw 阶段固定摸 2 **之后**内联钩（玉如意同位）：draw_cards +1 → take hand[0] → deck.push（确定性首张登记差异上游自选；置顶牌留待下位摸牌者——与上游同净效：手牌 +2、牌堆 -2） |
| 奥剑 aojian（Miles） | 出牌阶段弃 X 张手牌对攻击距离内至多 X 名各 1 伤（X=已损失体力；usable:1） | SwdAojian(tag 407)=ActivateSkill usable:1：SelectCard 1..X → TargetChoice 攻击距离 ≤1 至多 X 名 → 各 1 伤 |

## 勘误（入记忆）

- **kbolan 钩序**：+1 摸与置回必须在固定摸 2 **之后**——先置回会被固定摸立即重抽（首版序错实证推演）；后置序手牌净 +2 且置顶牌留堆顶，与上游完全同净效。

## 验证

- **定向回归** `swd_deferred_b98_kbolan_aojian`：拷贝栏跨回合推进至 Draw 完成后净 +2；奥剑弃 2（上限 X=2）→双目标各 1 伤+代价弃+限一次反例。
- **全量**：engine-native 815 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **131/184 映射，deferred 53→51**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
