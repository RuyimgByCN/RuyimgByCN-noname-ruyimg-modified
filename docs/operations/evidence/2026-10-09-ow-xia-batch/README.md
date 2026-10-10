# U05 xianjian 第四切片：瑕返灵（规则 v145，2026-10-09）

xianjian 第四武将切片：**瑕**（pal_xia，wu 3 女）——返灵（fanling，
他人失去体力后触发锁定每回合限一次）单技闭环；**xjzongyu 暂缓**（虚拟
useCard 飞镖——useCard({name:feibiao}) 随机 2 敌，虚拟用牌原语缺口第
四项同源）。xianjian 包 4/36 将入包（4 闭环）。前置：
[xianjian 第三切片](../2026-10-09-ow-zhaoliner-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js fanling）

- trigger global loseHpAfter forced usable:1——filter event.player !==
  player && player.isDamaged() → recover()。

## 改动（分开陈述）

- **代码已实现**：`TurnState.fanling_used: bool`（每回合限一次——链钩
  liangou2 usable:1 同款 TurnState bool；存档 **v15→v16**；两分支显式
  复位）+ `XianjianFanling`（tag 146，**EventKind::LoseHp** after 窗口
  触发=首个 LoseHp 消费者；LoseHp 帧有独立 kind+commit 路径（归有界发
  射）且 after 窗口照常计划触发）——filter（他人失体力有量 && 自己已
  受伤 && 本回合未发动）→ Act：置位+回复 1。
- **登记差异**：无新增（loseHpAfter=原生 LoseHp after 窗口一一对应；
  usable:1=TurnState bool 同链钩口径）。
- **登记暂缓**：xjzongyu（虚拟 useCard 飞镖）。
- **测试基建**：`Engine::debug_place_judge_id`（按 ID 移入判定区——跨
  包无名牌/双包建局用）。
- **转换包**：cards.json 增 瑕（wu 3 女，fanling mapping+xjzongyu 暂
  缓 reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`xia_fanling_recover_on_others_hp_
  loss`（归有界黑色判定→1 号失体力→0 号回复 1（3→4）+置位/满血后
  filter 收窄断言）。
- **全量通过**：engine-native **466 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v144 基线 465 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 144→145 + fanling_used 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：决斗失败是**伤害**非失体力（fanling 不触发——
  改走归有界延时判定失体力路径，双包建局+按 ID 摆判）；归有界牌名前
  缀 swd_（跨包无名牌→debug_place_judge_id 助手）。

## 剩余

- xianjian 其余 32 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
