# U05 xianjian 第十三切片：慕长兰掠影（规则 v154，2026-10-09）

xianjian 第十三武将切片：**慕长兰**（pal_muchanglan，wei 3 女）——掠
影（lueying，ShaResolve 内联钩每回合限一次）单技闭环；**feixia 暂缓**
（虚拟 useCard 杀——随机敌，虚拟用牌原语缺口）。xianjian 包 13/36 将
入包（13 闭环）。前置：
[xianjian 第十二切片](../2026-10-09-ow-longkui-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js lueying）

- shaBegin usable:1——step0 获得目标 he 随机一张；step1 另有其他有牌者
  →目标 chooseTarget（非自己/非来源/有牌）；step2 player.discardPlayer-
  Card(选定者, forced, he)。

## 改动（分开陈述）

- **代码已实现**：`TurnState.lueying_used: bool`（每回合限一次——存档
  **v19→v20**；两分支显式复位）+ `WudianLueying`（tag 161 目录标记技）
  + **ShaResolve 内联钩**（龙翔钩后）——`!asked` 分支持技判定+目标存
  活有牌+未发动 → 获得目标 he 随机一张（CardsGiven 日志）→ 另有其他有
  牌者则目标选人询问（asking_lueying/lueying_victim 两字段入 Sha 快
  照）→ 应答臂：来源随机弃受害者 he 一张 + **asked 复位**重开闪询问。
- **登记差异**：无（三步一一对应；受害者为随机弃=上游 discardPlayerCa-
  rd forced 随机）。
- **登记暂缓**：feixia（虚拟用牌）。
- **转换包**：cards.json 增 慕长兰（wei 3 女，lueying mapping+feixia
  暂缓 reason）+ npack 资产哈希重算。
- **编译守卫**：CardsMoved 日志不存在（CardsGiven）；engine 哈希 vi-
  cim cast u8；standard/engine 测试 new-call 补 2 参。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`muchanglan_lueying_gain_and_victim_
  discard`（目标选人询问 observer=目标/来源获得目标一张牌（杀已离手
  0+1=1）/受害者被随机弃 1/每回合置位/杀照常 4→3）。
- **全量通过**：engine-native **475 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v153 基线 474 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 153→154 + lueying_used/Sha 快照两字段入 canonical_
  hash——14 场景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 23 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
