# 2026-10-11 T11 星宿阶段2系绝红片：绝红（规则 v244）＋共工条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `14c2f756`）。上游依据：`boss_juehong`（5193：trigger `player phaseZhunbeiBegin` forced——敌方全体逐个：有牌者，有装备 → 装备全弃（discard es）；无装备 → `discardPlayerCard(target,"h",true)` 随机弃一张手牌）；携带者 boss_shuishengonggong 共工（987：姜|共工/shen/hp5/skills `[boss_shenyi, boss_juehong, boss_xuanlin3]`）。

## 建模

- **绝红**（查询型 Noop，`BOSS_JUEHONG`）单臂（kernel Judge begin 段——封冻同位，准备阶段=判定阶段开始锚点先例族）：持有者准备阶段——敌方全体（`is_boss_side` 异侧、存活）逐个：无牌（手+装备均空）跳过；有装备 → 全部装备撤技能+入弃堆；无装备 → 随机弃一张手牌（**随机→手牌首张确定性近似**，先例族登记）。
- **共工条目**：`BOSS_GENERALS` 尾追加（def 索引 21，hp5）`[神裔, 绝红]`；boss_xuanlin3 玄麟3=已处置关卡面，条目不含。

## 登记差异

- 随机弃牌 → 手牌首张确定性近似（先例族）。
- 上游 discardPlayerCard 走弃牌询问展示（true=强制无询问）——native 直接入弃堆等价。
- 势力 shen→群目录近似（先例族）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_shuishengonggong_juehong_strip_enemies` | 五段：①0 号装备全弃 ②2 号弃一张手牌（首张确定性：余牌=原序 [1..]）③无牌者（3 号）不触发 ④全程守恒 ⑤绝红实例+装备区空存档往返 |
| 既有回归 | boss 全滤 40/40；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 243→244 全 14 位移；回退 243 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 244 全量重录 2/2 绿 |
| 全量 | **590 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 243→**244**；存档不变 v57（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：明星三技（boss_qingzhu 庆祝/boss_jiazu 夹击/boss_jiding 急叮）或 boss_zaoyi 造异（玄冥/共工死亡触发系）。
