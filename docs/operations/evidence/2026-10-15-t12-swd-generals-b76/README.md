# T12 swd 武将面批㊱：少微/伽兰多/齐

- 日期：2026-10-15
- 范围：三将 partial 转正——**61/79 名**（上批 58 将+3）；新增 HostBehavior tag 359-361
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（huangyu/xianyin/pojian 逐段核验）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd少微 (shu hp3, 女) | 焰羽·少微面 huangyu（jianji 联动 defer） | SwdHuangyu(tag 359)=ActivateSkill usable:1：SelectCards 弃两张红色牌→虚拟炽羽袭（Chiyuxi AOE，targets=座位序全体其他，同 AOE push 口径） |
| swd伽兰多 (qun hp4, 男) | 摄引 xianyin（mailun 八效轮盘 defer） | SwdXianyin(tag 360)=ActivateSkill（无 limits）：座位序逐判定区有牌者——弃其判定牌全部+其交一张手牌（确定性首张登记差异上游自选） |
| swd齐 (qun hp3, 男) | 破茧 pojian（yaotong/heihuo defer） | SwdPojian(tag 361)=CardsDiscarded 锁定：自己手牌空→牌堆→弃堆寻首张装备牌——**实体暂入手牌→Equipped 帧→commit_equip 全托管**（take_hand_card+同栏替换+grant_equip_skills） |

## 设计要点（勘误）

- **pojian 装备路径**：直接操作 equips 向量+手动 grant 会绕过 commit_equip 托管面；正解=实体入手牌+emit Equipped 帧，由 commit_equip 完成 take_hand_card/同栏替换/授技全流程。
- **advance 阶段摸牌抽牌堆**：advance_to_play 过程中其他角色摸牌会 pop 牌堆顶（end）——压底目标实体必须在 advance **之后**布或改用「清空同区竞品+留原区」法。

## 验证

- **定向回归** `swd_generals_b76_huangyu_xianyin_pojian`：焰羽弃双红→炽羽袭全体其他各受 1 火伤；摄引判定区乐弃+目标交首张手牌入己方；破茧双拆桥拆空手牌→牌堆唯一坐骑自动装备。
- **全量**：engine-native 788 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **61/79 名**（upstream 79 将 183 技）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
