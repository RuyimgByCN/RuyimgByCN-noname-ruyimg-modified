# 2026-10-11 T11 boss 内容第三批续片：穷奇覆泰（规则 v212）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `57793a50`；boss_qiongqi 批续片按指令最小切片——仅覆泰单技，全场禁桃面登记下批）。上游依据：`mode/boss.js` `boss_futai`（5722：phaseZhunbeiBegin → 全体 `isDamaged()` 按座位序各 `recover()`）。

## 建模

- **覆泰**：触发技（`TriggerMeta::PhaseBegin` forced——**判定阶段开始锚点**，神智「原生无准备阶段」同口径先例）——`FutaiBehavior` filter（持有者的判定阶段 begin）+ `FutaiMachine` 一步：`seat_order_from(owner)` 全体存活受伤者各 `recover(1)`（无挂起）。
- 新 `MachineSnapshot::BossFutai { owner }` 变体：archive tag **62**/canonical_hash **63**（两空间独立）/快照校验/恢复臂 None/a0 样例尾+variant_index 60。
- FROZEN 尾加「覆泰」（67 名）。

## 登记差异

- 覆泰回血起点=穷奇座位环序（上游 sortBySeat 全局序——回血集合互不依赖，序对结果无影响，不另登记）。
- **覆泰2（全场禁桃）未落**：场上存有穷奇且非其回合时全场桃不可用/不可救——跨界濒死救援 `is_tao_like` 与 PlayCard 桃校验两面，下批单独落。
- 延妒(+switch) 未落（下批；switch 门控以隐贼 v210 先例同构）。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_qiongqi_futai_recover_wounded` | 穷奇判定阶段开始：0/2 号受伤各回 1、3 号满血不动、状态存档往返 |
| 既有回归 | 逐厄/混沌四技/烛阴/关卡回归不变；native_kernel 214/0；a0 双守卫过（61 变体） |
| 版本回退探针（212→211 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **559 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 211→**212**（存档不变 v48） |
