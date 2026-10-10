# U05 xianjian 第三十三批切片（规则 v175，2026-10-09）

苏媚（pal_sumei，shu 3 女）射影（sheying）+花好（huahu）双技闭环；
dujiang 毒牌种不在目录登记暂缓。前置：
[玄墨批](../2026-10-09-xianjian-xuanmo-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js sheying/huahu/dujiang）

- `sheying`：source damageAfter usable:1——伤害牌入弃堆（position "d"）
  → 判定黑→获得伤害牌（direct 无询问）。
- `huahu`：enable phaseUse limited selectTarget [1,∞]——contentBefore：
  awakenSkill+loseMaxHp+clearSkills；content：每目标 recover+changeHujia
  +draw。
- `dujiang`：phaseUse 弃黑牌→随机敌人获得 createCard("du")——毒牌种不
  在目录登记暂缓。

## 改动（分开陈述）

- **状态**：`TurnState.sheying_used: bool`（每回合 1 次=usable:1 同口
  径，Judge begin 清）+ `PlayerState.huahu_used: bool`（limited 一次
  性）。存档 v33→**v34**。
- **宿主行为**：`XianjianSheying`（tag 196，source Damage forced：
  cards 分流带入伤害牌→置回合位+判定黑（红桃/方片外）→弃堆回收伤害
  牌入手）+ `XianjianHuahu`（tag 197，phaseUse limited 多目标主动：
  TargetChoice 任意名其他角色→置一次性+失 1 体力上限+clearSkills 自己
  （SkillToggled 日志）→逐目标 Yield：各 recover 1+获 1 护甲+摸 1）
  ——六表+ALL 195→197。
- **登记暂缓**：dujiang（毒牌种不在目录，实体发现亦不可用）。
- **cards.json**：苏媚行（shu 3 女）+ rehash 两步；**版本** 174→175；
  a0 追加 196/197。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`sumei_sheying_black_judge_reclaim`
  （判定堆顶压黑→杀伤害牌回收+回合位；判定牌花色随 seed 分支断言）+
  `sumei_huahu_sacrifice_multi_target`（双目标舍身：上限 4→3+clear-
  Skills 自己+目标各回复/护甲/摸牌+一次性拒绝）。
- **全量通过**：engine-native **507 passed / 0 failed / 1 ignored**
  （v174 基线 505 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 174→175 + 双字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①判定牌花色随 seed——压顶（debug_set_deck_top_
  by）后仍被无中/摸牌扰动时用分支断言兜底②花好上限削减用 saturat-
  ing_sub+hp 钳制（i32/u32 类型边界）③本轮 host.rs 结构破损一次（自
  标臂插错 fn+孤儿体）——深度扫描器定位后按臂重建。

## 剩余

- 苏媚半闭环（sheying/huahu 闭环、dujiang 毒牌暂缓）。xianjian 其余
  8/36 将按缺口族；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
