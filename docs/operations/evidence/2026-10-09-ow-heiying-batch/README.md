# U05 ow 纵切批二十二：黑影隐匿（规则 v139，2026-10-09）

ow 纵切批二十二：**黑影**（ow_heiying，wei 3 女）——隐匿（yinshen，
弃牌触发锁定）单技闭环；**qinru/maichong 暂缓**（useCardToBegin 判定
+跨回合目标存储列表 qinru_turn 分档，maichong 随 qinru 链）。上游 25
将已闭环第 21 将。前置：
[ow 纵切批二十一](../2026-10-09-ow-zhaliya-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js yinshen +
noname/library/element/player.js tempHide + library/skill.js qianxing）

- `yinshen`：trigger loseEnd forced——filter 自己手牌无基本牌 &&
  event.cards 含 original=="h" 基本牌 → tempHide()。
- `tempHide`：addTempSkill("qianxing", {player:"phaseBeginStart"})——
  「潜行」锁定技 mod targetEnabled（player!=target false）=不能成为其
  他角色的卡牌目标，至自己下回合开始。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.yinshen_active: bool`（存档 **v12→v13**）
  + `WudianYinshen`（tag 138，CardsDiscarded 帧触发 forced=思继先例同
  位）——filter（弃置含基本牌（Sha/Shan/Tao/Jiu）&& 自己手牌无基本牌
  && 未置位）→ Act：置潜行；**目标门控**：PlayCard 目标校验并入
  jidong_active 同款（`yinshen_active || jidong_active` 一处条件）；解
  除=自己回合判定阶段开始（kernel Judge begin 清位块扩展——tempHide 至
  phaseBeginStart 同口径）。
- **登记差异**：上游 loseEnd 含所有失牌路径（original=="h" 区分），原
  生按弃置路径 CardsDiscarded 覆盖（给出/交换等失牌不触发——登记）。
- **登记暂缓**：qinru/maichong（跨回合列表）。
- **转换包**：cards.json 增 黑影（wei 3 女，yinshen mapping+双暂缓
  reason）+ npack 资产哈希重算。
- **过程小坑**：kernel 清位块锚点混用（commit_phase 在 kernel）——第一
  轮补丁半途失败后按文件分开落位。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`heiying_yinshen_untargetable_
  until_own_judge`（弃置基本牌→潜行置位/1 号回合杀 0 号 IllegalTargets/
  自己判定阶段解除）。
- **全量通过**：engine-native **460 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v138 基线 459 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 138→139 + yinshen_active 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：弃牌门在进入弃牌阶段的 NextPhase 即挂起（非再
  一次 NextPhase 后）；0 号收束需完整三段（Discard→End→收束）。

## 剩余

- ow 其余 3 武将（猎空/死神/D.Va）逐武将纵切；xianjian 36 武将；开局
  事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
