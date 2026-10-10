# U05 swd 第十六批切片（规则 v189，2026-10-10）

机关牌种基建+指路小虎（zhiluxiaohu）首切片——机关牌定义在上游
`card/swd/card.js`（非 xianjian），本批为 **swd 卡牌切片延续**与机
关牌族（御甲/偃师）解锁路径首格。虚拟杀原语第六次消费。前置：
[天符批](README.md)。

## 上游语义核验（源码为准，…/card/swd/card.js zhiluxiaohu）

- `zhiluxiaohu`：type:"jiguan"——enable=能用杀（filterCard sha）；
  selectTarget -1+filterTarget target===player+modTarget（自目标）；
  content=随机敌 randomGet→useCard({name:"sha"})（虚拟杀）+
  addSkill/removeSkill 相邻（AI 标记，原生略）。

## 改动（分开陈述）

- **牌种**：`CardKind::Jiguan` 编码 **132**（catalog ALL+key
  "jiguan"+archive 编解码）；swd 转换包 +swd_zhiluxiaohu ♣4（花色
  点数手补=护符同口径，上游 fullskin 无花色）+rehash 两步。
- **宿主行为**：`SwdZhiluxiaohuCard`（tag 216，卡牌机器=无懈窗口包
  装[流星火雨同款]，kernel kind 分派）——随机对一名其他存活角色虚
  拟杀+本体弃置（卡牌机器惯例）。**登记差异**：①敌方=全部其他角色
  ②上游 enable=能用杀门不校验。PlayCard 校验：自目标
  targets==[current]。
- **swd 存量回归修复**（新卡 ♣4 挤动发牌敏感路径=.deck-composition
  -test-traps 同款）：三处注入补三区 retain（deck+discard+全员手
  牌）+计数断言 25→26——**16/16 全绿**。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`swd_zhiluxiaohu_random_enemy_
  virtual_sha`（kind==Jiguan 断言+自目标使用→无懈 Pass→3 人局候选
  唯一敌虚拟杀 -1[seed 10065]→小虎入弃堆）。
- **全量通过**：engine-native **522 passed / 0 failed / 1 ignored**
  （v188 基线 521 +1；g1_shahai_swd 16/16 存量修复后全绿）；依赖包
  **63/0**（content-runtime 33 + engine-ai 29 + content-native-loader
  1）。
- **golden**：版本 188→189 + Jiguan 编码入 canonical_hash——14 场景
  全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 御甲/偃师续暂缓（需木甲人/勾魂锣等更多机关牌切片+计数基建）；
  dujiang 毒牌种独立评估；zhenfa 余量（长蛇/天覆/七星/鸟翔/破阵决
  deferred）；长期项不变（整包验收 0/37 等）。

## 改动文件

见 [source-hashes-v189.txt](source-hashes-v189.txt)（15 文件）+
[test-logs-v189.txt](test-logs-v189.txt)。
