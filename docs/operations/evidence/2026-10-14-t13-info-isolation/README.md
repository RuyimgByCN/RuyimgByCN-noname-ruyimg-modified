# T13 信息隔离跨模式验收（2026-10-14）

## 任务来源

[进度主稿 T13](../../../architecture/native-progress.md) 余项「全部模式/扩展信息隔离」的可执行切片：对已建模对局形态逐模式断言座位视图隔离语义 + 规划隔离内核面。

## 交付（crates/engine-native/tests/t13_info_isolation.rs 新增 7 测）

| 模式/面 | 断言 |
|---|---|
| 标准原型 | 逐座位矩阵：自己 `Own`/他人 `Hidden`；牌库或弃牌可见 |
| 身份局 | 他人手牌 Hidden（角色不因身份暴露） |
| 对决 2v2 | 队友手牌同样 Hidden（队伍关系不放宽可见性） |
| 斗地主 | 地主/农民两侧 Hidden 语义成立 |
| Boss 关卡 | 挑战者侧互相 Hidden（跨阵营一致） |
| 规划隔离（内核面） | `redact_opponent_hands` 确定性重分配：对手手牌**数量**保留、组成池内洗牌（搜索不使用真实暗牌归属）、自己手牌不动 |
| 扩展内容 | 欢乐卡牌扩展卡隔离语义与标准卡一致（Own/Hidden 不因内容来源放宽） |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **7/7** 绿——[日志](isolation-targeted.log) |
| engine-native 全量 | **711/0/0**（703→711，新增 7 测+1 试点文件迁移中的未用项清理）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 登记边界

- 「全部模式」覆盖当前已建模形态（原型/身份/对决/斗地主/Boss）；未建模模式（KOF 连续会话、国战完整版等）随各自模式批。
- 「扩展」覆盖已装载内容束；vm 技能的运行时信息边界由 rule-vm 查询白名单承载（O3/O4 防线），不重复断言。
