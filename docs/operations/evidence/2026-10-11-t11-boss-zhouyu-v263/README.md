# 2026-10-11 T11 群雄乱入周瑜片：火神/红莲·周瑜/仙音（规则 v263）＋周瑜条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续；本批开工前按用户指示先行提交了并行会话 D1–D3 的工作树（提交 `3808425f`，测试复核 616/0 绿）以释放热区）。上游依据：`boss_zhouyu`（周|瑜/shen/hp6/skills `[huoshen, boss_honglian, boss_xianyin]`，boss.js:1552）；`huoshen`@9761 / `boss_xianyin`@9782 / `boss_honglian`@9867。

## 建模

- **火神**（查询型 Noop，`BOSS_HUOSHEN`）：commit_damage **火伤分支直读臂**（招祸同位=damageBegin 层）——目标持有且存活 → `DamagePrevented` 日志+`recover(target,1)`+return（上游 `damageBegin1 cancel+recover()` 无参=1 同口径；取消后增幅不结算）。
- **红莲·周瑜**（查询型 Noop，`BOSS_ZYHONGLIAN`）：End begin 臂——摸 2+全部其他存活角色各 1 点火焰伤害（上游 draw(2)+逐个 `damage("fire")` 同口径）。
- **仙音**（查询型 Noop，`BOSS_XIANYIN`）：commit_cards_discarded 臂——非当前回合失去牌后判定：红→座位序首个其他存活角色失 1 体力（上游 loseEnd frequent judge 红 → chooseTarget loseHp；chooseTarget AI 态度面→首个确定性近似登记）。
- **周瑜条目**：`BOSS_GENERALS` 尾追加（def 索引 41，hp6，male）`[火神, 红莲·周瑜, 仙音]`；势力 shen→群目录近似。

## 同名改名映射（迅影批先例）

上游 `boss_honglian`（周瑜）与 `boss_honglianx`（毕方）**显示名同为「红莲」**（boss.js:10844/11190）。毕方红莲已以 `BOSS_HONGLIANX="红莲"` 落目录（FROZEN 81 名，冻结表只允许尾部追加不可改名）——周瑜红莲按迅影批改名映射先例以 **「红莲·周瑜」**（`BOSS_ZYHOLLIAN`）入目录，基准登记映射：原生「红莲·周瑜」↔上游 `boss_honglian`；原生「红莲」↔上游 `boss_honglianx`。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhouyu_huoshen_fire_immune_recover` | ①周瑜 4 血火伤 2→取消+回 1=5 ②普通伤害对照照常扣血 ③往返 |
| 新回归 `boss_zhouyu_honglian_end_and_xianyin` | ①红莲：End 摸 2+其他存活各 1 火伤 ②仙音负例：被拆+判定黑（预置黑桃）不失血 ③仙音正例：判定红（预置红桃）→首个其他存活失 1 ④往返 |
| 版本递增探针 | 普通批口径：263→262 回退复跑 golden **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 行级程序化重录 2/2 绿 |
| 全量 | **618 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 262→**263**；存档不变 v60 |

## 坑位实录

- **FROZEN 顺序与 def 表顺序不一致**：FROZEN 追加写成「火神/仙音/红莲·周瑜」而 def 表为「火神/红莲·周瑜/仙音」→ `standard_skill_ids_append_only` 名序断言拦（全量首跑 9/1 即失败）。修正=FROZEN 严格按 def 表顺序追加。
- **判定牌自然序不可控**：仙音负例首版未预置判定，自然判定恰红→假失败。负例须 `debug_set_deck_top_by` 预置黑桃。

## 遗留登记（本批新增）

- 火神免疫后 Damage 帧仍走 AfterTriggers（prevent 族同口径登记）；火神+招祸同持（无此条目组合）未建模双触发序。
- 仙音：loseEnd 全牌损失面→CardsDiscarded 面（胆术同口径）；chooseTarget AI→首个其他存活。
- 红莲·周瑜「对所有敌人」：上游 `game.filterPlayer` 语义下全场其他存活（Boss 模式周瑜无友方）=全部其他存活 ✓ 无偏差；群雄乱入其他将组队时的敌我划分随 boss 模式敌我机制（登记既有口径）。
- 下批候选：张角（天道/雷击[与青龙 BOSS_RELEIJI 同名改名映射]/极点，勘验已完备）、五官王/宋帝王/卞城王/泰山王/转轮王（勘验完备）、庞统/诸葛亮（重片专项）。
