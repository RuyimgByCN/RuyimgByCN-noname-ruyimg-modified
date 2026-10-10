# U05 xianjian 第十八武将切片（规则 v159，2026-10-09）

龙幽（pal_longyou，wei 4 男）月行（yuexing）+敏杀（minsha）双技闭环，
无暂缓本体。前置：
[xianjian 第十七切片](../2026-10-09-xianjian-jingtian-batch/README.md)。

## 候选选择（字节表评估）

剩余 19 将逐档核验上游：pal_xianqing 1991（xiaoyue roundStart 事件缺口
+自动 useCard 虚拟用牌+xiaoyue2 响应花色 mod）、pal_sumei 2018
（dujiang createCard 造牌「du」缺口+huahu clearSkills 结构性）、
pal_yueqi 2186（liguang moveCard 移动场上牌缺口+shiying dieBefore 死亡
拦截缺口）、pal_changqing 2219（luanjian viewAs 双杀转化+ctianfu 跨玩
家 storage 卡牌跟踪）、pal_murongziying 2508（qianfang 万箭 AOE 虚拟
用牌）、pal_liumengli 2540（tianxian mod 系+xjzhimeng 暗牌标记）——取
**pal_longyou 2622**（yuexing 有义从/链钩 computed_distance 状态修饰
先例管道，minsha 全现成原语；双技全闭环）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js yuexing/minsha）

- `yuexing`：enable phaseUse usable:1 filterTarget 其他角色——content
  双方 addTempSkill("yuexing2","phaseUseAfter") 互指 storage；yuexing2
  mod globalFrom：from.storage.yuexing2 存在时 distance-pure(from,to)
  +pure(partner,to)=出距以配对者座位为基数；phaseUseAfter=出牌阶段结束
  解除。
- `minsha`：enable phaseUse usable:1 selectCard 2（he 区）filterTarget
  体力>1 的其他角色——target 1 雷伤→game.filterPlayer(距 target≤1 且
  ≠target≠自己).sortBySeat(target) 各 randomDiscard（逐目标 redo）。

## 改动（分开陈述）

- **状态**：`PlayerState.yuexing_pair: Option<u8>`（双向互指；自己出
  牌阶段结束解除）。存档 v24→**v25**（PlayerState 段追加 opt_u8，位
  于 jiangcheng_longhuo 后）。
- **距离修饰**：`computed_distance` 增月行臂——a 持对偶时出距基数改用
  `seat_distance_alive(pair, b)`（义从/链钩/跳跃同区追加；**登记差
  异**：上游 mod 代换作用于 pure 距离后其余修饰照算，原生=仅替换座位
  基数、马匹等修饰照常叠加）。
- **解除钩**：commit_phase `!begin && phase==Play`——阶段归属者与配对
  者双向清除（蛊舞递减钩同区）。
- **宿主行为**：`XianjianYuexing`（tag 169，active：TargetChoice 发问
  →Act 收应答双向置对）+ `XianjianMinsha`（tag 170，active 三段：弃两
  张 he→目标选择→Act 双阶段循环——主目标队列逐目标雷伤（Yield 后重查
  存活，死亡者不结邻座=上游 isIn 同口径），每主目标结算后其距 1 内其
  他角色（除自己）按座位序随机弃一张（手牌优先退随机装备=蓝格怪衣同
  款登记差异）；self.card 兼「主目标结算中」标记）——六表+ALL
  168→170；双技 usable:1 走命令层 once_per_turn。
- **cards.json**：龙幽行（wei 4 男）：yuexing→host:xj_yuexing、
  minsha→host:xj_minsha（双闭环无暂缓）；rehash 两步重算。
- **版本**：STANDARD_RULESET_VERSION 158→159；a0_tag_registry 追加
  169/170。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`longyou_yuexing_distance_swap_
  and_clear`（computed_distance 直读：0→2 基线 2；激活指定 2 后 0→1
  以 2 座位为基数=1、2→3 以 0 座位为基数=1、双向互指；Play→Discard
  解除复原）+ `longyou_minsha_thunder_and_neighbor_discard`（弃两
  张→1 号雷伤 4→3→距 1 号 1 的 2 号随机弃 1（唯一候选桃入弃堆）、距
  2 的 3 号不弃）。
- **全量通过**：engine-native **483 passed / 0 failed / 1 ignored**
  （v158 基线 481 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 158→159 + yuexing_pair 入 canonical_hash——14 场景
  全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：主动技首步无应答可读的坑第三次出现（景天批已登
  记，月行初稿又犯——TargetChoice 首臂读 answer panic；定式=首臂发问、
  次臂收应答）。

## 剩余

- xianjian 其余 18/36 将逐武将纵切；本轮评估登记的缺口族：xianqing
  （roundStart+虚拟用牌）、sumei（createCard+clearSkills）、yueqi
  （moveCard+dieBefore）、changqing（viewAs+跨玩家 storage）、
  murongziying（AOE 虚拟用牌）、liumengli（mod 系+暗牌标记）；长期项
  不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
