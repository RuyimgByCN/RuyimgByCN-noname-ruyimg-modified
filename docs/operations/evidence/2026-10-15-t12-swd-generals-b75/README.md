# T12 swd 武将面批㉟：凤钰/轩辕剑仙/慕月

- 日期：2026-10-15
- 范围：三将 partial 转正——**58/79 名**（上批 55 将+3）；新增 HostBehavior tag 356-358
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（shangxi/huajian/xingzhui 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd凤钰 (shu hp4, 男) | 袭西·凤钰面 shangxi（fzhenwei respondEnd 弃牌面 defer） | SwdShangxi(tag 356)：Phase{Play,begin} 直发窗口——SelectCard 弃一张 he→TargetChoice 攻击距离内且体力不小于自己者→1 伤 |
| swd轩辕剑仙 (qun hp4, 男) | 花间·剑仙面 huajian（pozhou 计数/xuanyuan 召剑 defer） | SwdHuajian(tag 357)：Phase{Play,begin=false}（出牌阶段结束）直发窗口——SelectCard 弃一张 he→TargetChoice 杀合法目标→虚拟杀 |
| swd慕月 (wei hp3, 女) | 星坠·慕月面 xingzhui（灵心? 否——凌仙/授因 defer） | SwdXingzhui(tag 358)=ActivateSkill usable:1：SelectCard 弃一张牌（类别锚）→TargetChoice→目标弃同类别一张或受 1 伤（类别三分类 basic/equip/其他登记差异上游 get.type 同类） |

## 设计要点

- **Play-begin 触发技测试法**：advance_to_play 会停在 Play（begin 帧已过窗）——须推进到 Judge/Draw 停，再手动 NextPhase 入 Play 并**立即**取窗（advance 的自动应答会吞窗）。
- 类别锚三分：锚牌 is_basic→基本候选 / is_equip→装备候选 / 其余（锦囊）→非装备非基本候选（上游 get.type(card,"trick") 三分同构登记）。

## 验证

- **定向回归** `swd_generals_b75_shangxi_huajian_xingzhui`：袭西弃牌→p1（距离内 hp≥自己）受 1、p3 未选不变；花间阶段末弃牌→虚拟杀 1 伤；星坠弃基本牌锚→目标弃同类别基本牌、其余保留。
- **全量**：engine-native 787 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **58/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
