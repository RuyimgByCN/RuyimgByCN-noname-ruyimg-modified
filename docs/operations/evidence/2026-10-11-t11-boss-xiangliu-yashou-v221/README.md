# 2026-10-11 T11 四神兽系首片：相柳妖兽（规则 v221）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `e24723e0`；四神兽系首切片——妖兽共有技单技+相柳条目）。上游依据：`mode/boss.js` `boss_yaoshou`（4735：mod globalFrom distance-2）与 `boss_xiangliu`（889：hp25/五技）。

## 建模

- `BOSS_GENERALS` 增**相柳**（群/hp25/`[妖兽]`）——**登记**：毒妒（受伤者获蛇毒标记+回合开始弃 X 或失 X[标记衰减链 storage]）/兽九（手牌上限恒 9+出牌/结束补至 9+跳过摸牌）/邪酬(+switch)（hp≤半激活+他人桃酒获蛇毒标记）未落随下批——三技均涉 storage 标记存储面。
- **妖兽**：权限型 Noop+`computed_distance` 距离链（凶兽-1 同款，-2）。
- FROZEN 尾加「妖兽」（77 名）。

## 连带修正（boss 段动态尾段的既有测试假设过期）

boss 段（现 7 将）位于扩展段后——g1_d3 两处「base+7 越界」硬编码假设失效（base+7 现落 boss 段命中相柳）：越界断言改 `general_count()` 动态总尾、越界槽恢复改 `general_count()+7`。**教训**：boss 段为动态尾段，任何「扩展段后越界」的测试假设须经 general_count()。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `boss_xiangliu_yaoshou_distance_minus_two` | 相柳距离-2（2-2 钳下限）；他人间距离不受影响 |
| 连带 | g1_d3 两断言按动态总尾修正后 4/0 |
| 既有回归 | 梼杌四技/书精/饕餮/穷奇/混沌/烛阴/关卡全部不变；native_kernel 223/0 |
| 版本回退探针（221→220 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **568 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 220→**221**（存档不变 v49） |
