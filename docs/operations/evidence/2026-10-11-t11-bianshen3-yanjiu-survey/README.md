# 2026-10-11 T11 bianshen3 勘验处置＋炎狱计数器勘验结论＋first=0 交互勘验（无版本递增）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `3a2ea8d5`；阴间系收尾后勘验批）。

## 一、bianshen3 勘验处置（不落）

`boss_bianshen3`（8747：dieBegin hide + global dieAfter `game.changeBoss([boss_baiwuchang, boss_heiwuchang] 随机)`——牛头/马面死亡换 boss）与 `boss_bianshen3x`：**bianshen4 死亡 changeBoss 同族**（隐藏链八技+bianshen4 勘验批已立处置口径）——关卡流程面不落。牛头/马面条目不含 bianshen3（v255 已只列已转换技）。

## 二、炎狱跨回合计数器勘验结论（登记差异，不修正）

上游 `boss_yanyu`（毕方）：`global phaseBegin` + **`getIndex: 3`**——每事件触发三份（每次其他角色回合开始判定 3 次，每份非红 → 1 火伤），无跨回合计数。
原生 v229 口径：`TurnState.yanyu_count` 回合级计数器封顶 3（每回合开始判定 1 次，跨回合累计至 3 后不再触发）。
**差异**：上游=每事件 3 判定；原生=每回合 1 判定×跨回合累计 3。行为分叉（上游每轮最多 3 伤 vs 原生三轮各 1 伤共 3 伤——总量近似但时序分布不同）。
**不修正理由**：炎狱计数器（TurnState.yanyu_count）已成 v229 存档面（v59 编码），修正触发次数分布=语义变化需重新勘验红莲/炎狱叠加血量算术（本次勘验中因红莲每回合+1 火伤叠加、挑战者 AI 手牌随机行为与 matchOver 连锁，血量精确断言不稳定）。登记差异保留原生口径，待炎狱专项批修正。

## 三、first=0 交互勘验（不构成缺陷）

`new_boss_stage(seed, seats, boss, chongzheng, boss_cards, player_cards, first=0)`：first=0 时 0 号（挑战者）先手。虚失翻面臂（v225）锚=弃牌阶段开始——**持有者自己的弃牌阶段**，与 first 无交互；先前登记的「自动推进交互」实为 v248 前 old test 的驱动链误置（跨回合裸推进），非引擎缺陷。回归 `boss_zhuyan_xushi_flip_and_aoe`（v225 起持续绿）覆盖。结论：**撤销该待查**（无缺陷可修，登记沿革保留）。

## 验证

| 项 | 结果 |
|---|---|
| 引擎改动 | 无（勘验登记批） |
| 版本/存档/golden | 规则 257/存档 v59/14 golden 均不动 |
| 全量 | 本批无代码改动，沿用 v257 实测 **604/0/1**（同源码） |
| Gate 1b | cargo 真实 exit 0（同源码复跑） |
