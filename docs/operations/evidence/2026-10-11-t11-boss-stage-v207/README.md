# 2026-10-11 T11 Boss 关卡骨架（规则 v207）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `e0e154e4`；承接 [boss 专属技能包首切片评估](../../2026-10-11-t9-lx-trace-survey/README.md)——消费面先行）。上游依据：`mode/boss.js` 各关卡 `bossinfo.gameDraw`（如穷奇 boss 8/玩家 4）与 `bossinfo.loopFirst`（回合起点座位）。

## 建模

- `OpeningPlan`（引擎私有建局面枚举，泛化 v205 的斗地主专用参数）：`Standard | DoudizhuBinglin | BossStage { boss_cards, player_cards, first }`——发牌数与回合起点为**建局输入不入模式编码**（已进状态与 canonical_hash，存档格式不变 v48）。
- `Engine::new_boss_stage(seed, seats, boss, chongzheng, boss_cards, player_cards, first)`：关卡开局配置；`first` 越界回退座位 0。回合起点=state 构造后覆盖 `turn.current`。
- `loopFirst` 函数面以显式座位参数承载（关卡函数由调用方按上游语义求值）。

## 登记差异

- `bossinfo.minion`（随从配置）/`control`/`init` 关卡脚本、`loopType==2` 交替回合序：内容面，随 boss 专属技能包逐关卡推进。
- `loopFirst` 的「函数」形态以座位参数近似（上游可动态求值）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_stage_opening_and_first_actor` | 关卡面 boss 8/挑战者 4、起点=指定座位 2、存档往返一致、非法起点回退 0、标准 `new_boss` 回归不变（各 4 张+起点 0） |
| 既有 Boss 回归 | `boss_mode_chongzheng_revival_and_victory`、`boss_mode_add_fellow_seat_expansion` 不变通过（全量含） |
| 版本回退探针（207→206 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；Boss 不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **554 通过 / 0 失败 / 1 ignore**（552+2 拆分断言段，2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 206→**207**（存档不变 v48） |

## 下一切片

boss_zhuyin（hp4 单技 boss_xiongshou 凶兽）+ boss_xiongshou 转换落包（消费面已就绪：`new_boss_stage` + 将装配 debug 面）。
