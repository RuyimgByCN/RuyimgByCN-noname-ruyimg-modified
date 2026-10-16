# T12 swd deferred 消解批 92：黑火/真威/遁行

- 日期：2026-10-16
- 范围：三技 deferred 消解（tag 392-394）——**118/184 技条目已映射**（68→65 项 deferred）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（heihuo/fzhenwei/dunxing 逐段核验）

## 三技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 黑火 heihuo（齐） | 出牌阶段弃一张装备牌，手牌数加倍；若因此 ≥8 立即自受 3 点火伤且本回合不能再发动 | SwdHeihuo(tag 392)=ActivateSkill usable:1：SelectCard 弃装备（he）→摸等同弃后手牌数→≥8 自发 3 火伤 |
| 真威 fzhenwei（凤钰） | 自己回合内其他角色打出的卡牌（入弃堆）→ 可交给除该角色外的任意一名角色 | SwdFzhenwei(tag 393)=CardResponded optional：filter（响应者≠自己+自己回合+牌在弃堆）→确认→TargetChoice（≠响应者）→弃堆牌入手（CardsGiven） |
| 遁行 dunxing（虎魄） | 成为其他角色卡牌目标时，可弃一张牌并判定，非红桃则取消之 | SwdDunxing(tag 394)=CardUsed 目标侧 optional：确认→SelectCard 弃一张 he→内联判定（同铁骑口径）非红桃→cancelled 写回+桌面归还使用者手牌（工神同款） |

## 登记差异

1. **真威**：上游 respondEnd 的 cards 可多张（连环响应），原生单牌载荷逐次触发（等价面）；「打出」限定为响应牌（打出响应=respond），使用牌不触发。
2. **遁行**：判定内联确定性（无鬼道改判窗，同渔人/神鼎口径）；对延时锦囊自置不触发（player≠owner 门）。
3. **黑火**：加倍基数=弃装备后的手牌数（上游 countCards("h") 在弃置后求值同口径）。

## 验证

- **定向回归** `swd_deferred_b92_heihuo_fzhenwei_dunxing`：黑火弃装备手牌 2→4（<8 无伤）+限一次反例；真威响应者 p1 不在候选+闪交给 p3；遁行非红桃判定杀取消（无伤+代价弃+杀归还使用者手牌）。
- **全量**：engine-native 809 passed / 0 failed（golden 零扰动）。
- **Gate 1b**：exit 0。

## 结论

- **118/184 技条目已映射，deferred 68→65**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
