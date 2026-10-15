# T12 swd 武将面批㉞：海大胖/麦星/冯天凌

- 日期：2026-10-15
- 范围：三将 partial 转正——**55/79 名**（上批 52 将+3）；新增 HostBehavior tag 353-355
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（bingjian/shending/guiyan 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd海大胖 (wu hp3, 女) | 冰箭 bingjian（rumeng 阶段跳过 defer） | SwdBingjian(tag 353)=ActivateSkill usable:1：SelectCard 弃黑杀→TargetChoice 有手牌目标→**亮手牌（弃全部闪）或 1 雷伤**（thunder） |
| swd麦星 (wu hp3, 男) | 神鼎·麦星面 shending（toudan 链式火伤 defer） | SwdShending(tag 354)=ActivateSkill usable:1：确定性判定四分支——红桃回 1/方块摸 1/梅花随机敌 randomDiscard 1/黑桃无（内联口径同铁骑） |
| swd冯天凌 (shu hp4, 男) | 鬼眼 guiyan（guiyan2 濒死复活 defer） | SwdGuiyan(tag 355)=ActivateSkill usable:1：TargetChoice 有梅花手牌目标→SelectCards 取其一张梅花获得 |

## 设计要点（勘误）

- **多段 activate 机器每段写 self.target**：TargetChoice 消费段必须 `self.target = Some(t)` 再前移 stage——鬼眼首版漏写致 SelectCard 段 self.target=None 静默 Done（ DBG 实证 h0 不变）；与鸣凤/对诗同款，多段机铁律第 N 犯。
- 神鼎 Club 分支=随机敌 randomDiscard（RNG 同上游 randomGet）。

## 验证

- **定向回归** `swd_generals_b74_bingjian_shending_guiyan`：冰箭黑杀弃→闪全弃（p1 双闪清空）；神鼎红桃判定回 1+限一次反例；鬼眼目标窗→取牌窗→梅花换手。
- **全量**：engine-native 786 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **55/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
