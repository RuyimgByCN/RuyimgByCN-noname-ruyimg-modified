# 2026-10-11 T11 星宿阶段2系星夏片：星夏（规则 v242/存档 v57）＋祝融条目＋赤翼/白翼处置

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `914e1bfc`）。上游依据：`boss_xingxia`（6940：enable phaseUse 主动技——filter 场上有 boss_yanling+storage 轮次门控；content 对 target `damage(2,"fire")`；contentAfter storage=round → chooseTarget 一名敌方 → 其 chooseToDiscard 红牌[he 位]或受 1 火伤，两段均可放弃）；携带者 boss_huoshenzhurong 火神祝融（1020：姜|重黎/shen/hp5/skills `[boss_shenyi, boss_xingxia, boss_chiyan3]`）。

## 赤翼/白翼族属勘验处置（登记不落）

- **boss_chiyi 赤翼**（7105，炎帝）：phaseZhunbeiBegin 轮次 [3,5,7] 编排——3=敌方全体授 boss_chiyi2 标记技（受伤+1）、5=全场各 1 火伤、7=焰灵 5 火伤——**青翼同族**（轮次编排+跨将授技，关卡耦合）。
- **boss_baiyi 白翼**（5325，句芒）：三子技全硬绑轮次——roundStart round==5 敌方各弃 2 / rounds<3 敌方摸牌 -1 / rounds<7 雷伤免疫——**青翼同族**（轮次门控）。
- 二者按青翼先例处置登记（轮次编排族=关卡耦合面），不落内核；炎帝/句芒条目暂不入库（其可承接内容技待后续批）。

## 星夏建模

- **星夏**（`XingxiaBehavior`+`XingxiaMachine`，主动技 `ActivateSkill` 直调——不臣先例第二例）：
  - **损焰灵线**：对场上焰灵（BOSS_GENERALS 段按名查目录+座位装配匹配）`emit Damage(2, Fire)`；`xingxia_used_round=round` 登记。无敌方可追击 → 即止。
  - **追击线**（可放弃）：SelectPlayers 异侧存活（min 0/allow_cancel）→ 选中者弃一张红色牌（手牌+装备，he 位；拒绝/无红牌 → 1 点火伤）。
  - **间隔门控**：`xingxia_used_round`（`round-used<2 → InvalidChoice`）。
- **快照变体 `BossXingxia`**（archive tag 67/机哈希 68/a0 65；stage 0/1/2/3——弃红询问由阶段 2 直接挂出；active_ask 按 stage 1/3 派生候选）。
- **xingxia_used_round 新字段**（`PlayerState`，**存档 56→57**；编解码双表；canonical_hash 入参——字段新增批）。
- **祝融条目**：`BOSS_GENERALS` 尾追加（def 索引 18，hp5）`[神裔, 星夏]`。

## 登记差异

- 上游 selectTarget -1（焰灵目标数不限）——原生全场唯一焰灵座位即唯一目标（等价承载）。
- 上游 chooseTarget 敌方无 line/show 面（UI）不落。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- **min=0 可放弃询问绝不能 drain_choices**：空应答合法 → drain 把「弃红询问/追击询问」静默吞成「放弃」分支（本批回归三次踩中：激活后/选敌后/转敌后的三处过早 drain）。教训：**drain_choices 只适用于 min≥1 或无 min 约束的强制询问**；可放弃询问必须显式作答。
- 死常量（`XINGXIA_STAGE_ASK_DISCARD`——机器从阶段 2 直接挂弃红询问，无独立挂起码）被 Gate 1b `-D warnings` 拦截；弃牌完成态改用码 3。**Gate 1b 的 exit 必须取 cargo 真实退出码**（管道 tail 会吞非零退出——首跑假 exit=0 被抓出）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_huoshenzhurong_xingxia_yanling_burn` | 六段：①损焰灵线（焰灵 4→2+轮次登记）②追击线（选 2 号→弃红免伤）③同轮再发动被拒 ④放弃追击（空应答后无后续询问）⑤弃牌拒/无红牌→1 火伤 ⑥xingxia_used_round 存档 v57 往返+全程守恒 |
| 既有回归 | boss 全滤 38/38；a0 样例 2/2；机械审计：无多重注册/零属性清零 |
| 版本/字段探针 | 字段批口径：241+新代码 14 全位移（字段源）+版本源；重录后 2/2 绿 |
| 全量 | **588 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | **cargo 真实 exit 0**（死常量清理后复跑；管道假 exit 教训入档） |
| 版本 | 规则 241→**242**；存档 56→**57** |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：星宿阶段 2 收尾勘察（赤翼/白翼已处置；剩余=炎帝/句芒条目是否以纯神裔形态入库、共工[水神]系 boss_shui? 技、boss_mingxingzhu 明星系[星囚授 jiding 面的依赖将]）。
