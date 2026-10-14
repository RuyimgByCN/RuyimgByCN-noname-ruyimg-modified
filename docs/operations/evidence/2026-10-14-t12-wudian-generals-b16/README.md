# T12 玩点论杀批 16：良策余获「选目标近似升级」勘察定案（2026-10-14）

## 勘察结论（结构性不可达，回滚实施）

批 15 曾登记「良策选目标近似升级」为后续项。本批实施后**勘察发现结构性
不可达**：原生五谷机器亮牌数 = `min(存活数, deck 剩余)`、择牌数 = 存活数
——**存活角色数 ≥ 亮牌数恒成立 → 永无剩余**（`finish_wugu` 时 `cards` 恒
空）。RemainGive 阶段（SelectPlayers 目标选择机+快照字段+archive 编解码
四处接线）为**不可达代码**，已全量回滚（kernel/snap/archive/engine/a0
fixture 六处）。

批 15 的「归持有者本人」近似在**可达面上与「可选目标」等价**（无剩余时
均无效果）——维持该近似并升级登记为**结构性等价**。上游剩余场景（多亮
牌/减员五谷）原生不建模，随五谷 Remained 子事件面整体挂账。

## 实施与回滚记录

- 实施触及：kernel WuguMachine RemainGive 阶段+step_remain_give 方法、
  snap 变体字段、archive enc/dec、engine 哈希模式、a0 fixture——六处
  接线完整后由测试暴露不可达（无剩余→选择询问不出现）。
- 回滚六处+批 15 测试余获段改结构性登记注释。

## 实测记录

| 项 | 结果 |
|---|---|
| 回滚后 | g1_wudian **122/122**、engine-native **744/0/0**、Gate 1b exit 0——[日志](engine-native-tests.log)/[Gate](gate1b-all-targets.log) |
| golden | 不变（回滚恢复批 15 基线） |

## 登记边界

- 五谷 Remained 子事件面整体挂账（含坚壁 wugu 分支、良策选目标）——需
  「五谷多亮牌」原语立项。
- 玩点论杀剩余暂缓（3）不变：仇渡（deferred 卡）、夺启/佑赞（存量）。
