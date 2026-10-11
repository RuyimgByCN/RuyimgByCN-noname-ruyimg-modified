# 2026-10-11 T11 相柳续片：兽九（规则 v222）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `6fcb0e71`；相柳续片按指令最小切片——仅兽九单技）。上游依据：`mode/boss.js` `boss_jiushou`（5899 附近：mod maxHandcard `num-hp+9`；phaseUseBegin/phaseJieshuBegin `draw(9-count)`；phaseDraw cancel）。

## 建模

- **兽九**三面（权限型 Noop+查询）：
  1. **手牌上限恒 9**（`hand_limit` 函数分支：兽九覆盖基数→9——上游基数 hp-hp+9=9 同口径；他人不受影响）；
  2. **跳过摸牌阶段**（Draw 臂：持技者不摸，上游 phaseDraw cancel 同口径）；
  3. **出牌阶段开始/回合结束补至 9**（Play/End begin 臂：`draw(9-现有)`）。
- FROZEN 尾加「兽九」（78 名）。

## 登记差异

- 毒妒（受击者获蛇毒标记+其回合开始弃 X 或失 X[标记衰减]）与邪酬(+switch)（hp≤半激活+他人桃酒获标记）涉跨回合标记存储（PlayerState 新字段+存档 v50+弃/失选择询问）——下批一并落。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_xiangliu_jiushou_limit_nine_and_refill` | 上限恒 9（hp25 时基数 25 被覆盖；他人 hp4 上限 4）；出牌阶段补至 9（跳过摸牌后仍 9）；End 补至 9（3 张→9） |
| 既有回归 | 相柳妖兽/梼杌四技/书精/四凶全家族/关卡全部不变；native_kernel 224/0 |
| 版本回退探针（222→221 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **569 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 221→**222**（存档不变 v49） |
