# U05 xianjian 第四十三批切片（规则 v185，2026-10-10）

越今朝（pal_yuejinzhao）**整将入包**（33→34 将，缺 2：长卿/居十
方）+影锋（xjyingfeng）单技闭环——雾魂（ywuhun）双引擎缺口暂缓登
记=越今朝半闭环。虚拟 AOE/虚拟杀原语第四次消费。前置：
[天弦批](../2026-10-10-xianjian-tianxian-batch/README.md)。

## 未入包 3 将评估（本批选型依据）

- **长卿**（乱剑+天符）：乱剑=双杀 viewAs 复合+50% 增伤可单切；天
  符=闪置武将牌上+延迟雷伤（挂附标记）中等。
- **越今朝**（雾魂+影锋）：影锋=迅影基建直接复用（最小）；雾魂=
  额外回合[ADR0646 引擎缺口]+全局牌复原双缺口。**本批选越今朝**。
- **居十方**（御甲+械磐+偃师）：御甲/偃师依赖「机关牌」种=目录缺
  口（dujiang 同族）；械磐=失去最后基本牌→实体发现获得随机装备可
  单切。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xjyingfeng）

- `xjyingfeng`：player useCardAfter forced——filter=card=sha &&
  parent.name!=="xjyingfeng"（防递归）&& 存在敌方非本杀目标且
  canUse；content=敌方去掉本杀目标→randomGet→useCard({name:"sha"})
  （虚拟杀，addExpose 0.2=AI 表现层）。

## 改动（分开陈述）

- **宿主行为**：`XianjianXjyingfeng`（tag 209，CardUsed
  AfterTriggers forced）——filter（自己用杀+链挡旗标未置+候选存
  在[其他角色-本杀目标，存活+距离内]）→step：RNG 随机候选→
  use_virtual_card 虚拟杀（牌堆底载体完整管线）。**登记差异**：①
  敌方=全部其他角色（prototype 无阵营）②上游 canUse 简化为距离口
  径③同回合多张真实杀仅首次触发。
- **防递归实测修正**：真杀 CardUseData.use_as 实测本为
  `Some(Sha)`（转化声明普遍存在）——use_as 挡递归方案不可行。改为
  `TurnState.yingfeng_chain` 链挡旗标（影锋机器 emit 前置位→虚拟
  杀的 AfterTriggers filter 拒→Judge begin 清）——上游 parent 检
  查的登记差异等价（差异③即由此）。
- **状态**：`TurnState.yingfeng_chain: bool`（入 canonical_hash+
  存档字节）。存档 v36→**v37**。
- **cards.json**：越今朝整将条目（wei/4 血/male；雾魂 null+理由登
  记、影锋 host:xj_xjyingfeng）+ rehash 两步；**版本** 184→185；a0
  追加 209。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`yuejinzhao_xjyingfeng_random_
  virtual_sha`（3 人局：真杀目标 1[Pass -1]→影锋自动虚拟杀候选唯
  一的 2 号[Pass -1]；hp1=3 未再扣=链挡防递归）。
- **全量通过**：engine-native **518 passed / 0 failed / 1 ignored**
  （v184 基线 517 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 184→185 + yingfeng_chain 入 canonical_hash——14
  场景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 越今朝半闭环（雾魂=额外回合+全局牌复原双引擎缺口）。未入包 2 将
  （长卿乱剑/天符、居十方械磐可单切，御甲/偃师=机关牌种缺口）；
  长期项不变（zhenfa 五阵法、整包验收 0/37 等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
