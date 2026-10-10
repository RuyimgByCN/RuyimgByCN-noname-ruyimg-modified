# 2026-09-23 第 30 轮：玩点论杀·诱敌深入响应转化（规则版本 41）

按玩点论杀包数据登记的 reason 增补**诱敌深入响应转化原语**
（`CardKind::Youdishenru`）并扩展玩点论杀切片至 **14/21**。
`STANDARD_RULESET_VERSION` **40 → 41**（Youdishenru 牌种；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **308 passed / 0 failed**（第 29 轮 308 → 含诱敌深入合并回归）。全工作区 4,875 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v41 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节（诱敌深入·响应转化）

- 成为杀目标时，响应选项并入手牌中的诱敌深入（`ShaResolveMachine`
  respond_shan 选项构建 + snap.rs 恢复派生同步）。
- 打出后：杀无效（目标不掉血），来源受 1 点反噬伤害（`Damage` 事件，
  牌归因 = 诱敌深入）。
- **登记差异**：上游为"来源被强制连用杀直至断链"多步循环 + 临时技
  `youdishenru` 存储标记；原生简化为单次反噬，无连用循环。
- 主动使用被拒（`WrongPhase`）——仅作为杀的响应打出。

## 包数据同步

wudian 包 cards 11→14（+诱敌深入×3，diamond 9 / spade 11 / spade 12），
deferred 仅剩调兵遣将×2（多人方向选择）。

## 回归

`youdishenru_cancels_sha_with_reprisal`（g1_wudian）：杀→诱敌深入响应 →
目标不掉血、来源受 1 点反噬、牌已消耗。

## 边界（如实登记）

- 玩点论杀切片 **14/21**（暂缓 7 张：调兵遣将 2 / 三尖外原语已补 /
  基础牌与铁索已转）；武将技能映射 2/11。
- 整包验收仍 **0/37**。
