# 2026-10-11 T11 隐藏链八技整族处置（赤炎2/玄麟3 等）——纯关卡流程面，不落内核

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `7d72df36`：朱雀续片 boss_chiyan2 赤炎；连带玄武续片 boss_xuanlin3 玄麟同族勘察）。本批为**处置登记批**：上游八技均为同构的关卡流程面，无引擎内容面——按「无语义变化不递增版本」先例（versus 4v4 固化批）只登记不落包。

## 上游勘察（boss.js 行号实测）

| 技能 | 行号 | 结构 | 链向 |
|---|---|---|---|
| boss_qingmu2 青木2 | 6253 | dieBegin hide + dieAfter 换位/changeBoss | boss_qinglong 阶段 1→2 |
| boss_baimang2 白芒2 | 6633 | 同上 | → boss_jinshenrushou（金神蓐收） |
| boss_chiyan2 赤炎2 | 6071 | 同上（+全场回满+addFellow 炎灵+复活已死挑战者） | → boss_huoshenzhurong（火神祝融） |
| boss_xuanlin2 玄麟2 | 6435 | 同上 | → 玄武阶段 2 |
| boss_qingmu3 青木3 | 6342 | dieBegin hide（本体+前后位三人）+ dieAfter | 青龙阶段 2→3 |
| boss_baimang3 白芒3 | 6722 | 同上（三人 hide） | 白虎阶段 2→3 |
| boss_chiyan3 赤炎3 | 6160 | 同上（三人 hide） | 朱雀阶段 2→3 |
| boss_xuanlin3 玄麟3 | 6523 | 同上 | 玄武阶段 2→3 |

八技共性：`charlotte+silent+fixed`、trigger `player dieBegin`（filter `player==game.boss`）→ `player.hide()`；配套 `*x` global `dieAfter`（priority -10）→ `changeSeat(6)/nextSeat.changeSeat(3)`、`game.changeBoss(<下一阶段武将>)`、阶段补给（回满/直授/addBossFellow/复活）。

## 处置依据

- hide（视觉隐藏）/changeSeat（观战位移）/changeBoss（换将）/addBossFellow（阶段援军入场）/跨阶段复活均为**关卡系统流程面**——Boss 模式消费侧缺口「无选将/关卡系统（裸骨架）」既有登记（T11 v207 勘察），引擎不承载为既定边界（同 KOF 多局：宿主层承载）。
- 四象隐藏武将条目（v233/v235/v236/v237 落包）本就只列已转换技，八技不在条目内——无目录/行为影响，**无语义变化，规则版本不递增（237 不变）、存档不变 v54、golden 不动**。

## 下批候选勘察（同族真实内容技，已读上游）

- **boss_shenen 神恩**（6872，三处阶段 2 武将共用）：boss 侧杀/牌无距离限制（`side → targetInRange=true`）+挑战者侧手牌上限 +1 与摸牌 +1（`!side → maxHandcard+1 / phaseDrawBegin num++`）——三面全查询型，最小下一片。
- **boss_xingqiu 星囚**（5485）：phaseUseBegin 交替横置全部敌方（连环）+ 对 boss_mingxingzhu 授 boss_jiding——依赖连环（link）机与跨将授技，较重。
- **boss_buchun 不臣**（7157）：出牌阶段失体力复活场上已死亡的书精（阶段流程+复活，依赖死亡区遍历）。

## 验证

| 项 | 结果 |
|---|---|
| 引擎改动 | 无（纯处置登记批） |
| 版本/存档/golden | 规则 237 不变、存档 v54 不变、14 golden 不动（无语义变化先例） |
| 链接检查 | 本 README 与基准/主稿互链有效（脚本核验） |
| 全量 | 本批无代码改动，沿用 v237 批实测 **583/0/1**（同源码） |
