# NC08 第⑦层：身份局**开局差异的上游判定**（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：`nc08_cross_line` 登记的 4 处开局差异**已按上游源码判定**：**原生与上游一致的两项**、**旧线偏差一项**、**原生缺「准备阶段」一项（登记）**

## 1 上游依据（归档快照，逐字引用）

`archive/upstream/noname/apps/core/mode/identity.js`（片段见
[upstream-identity-start.js.txt](upstream-identity-start.js.txt)）：

```js
event.beginner = _status.firstAct2 || game.zhong || game.zhu || _status.firstAct || game.me;
game.gameDraw(event.beginner, player => {
    if (_status.mode == "purple" && player.seatNum > 5) { return 5; }
    return 4;                     // ← 每人 4 张（含主公）
});
...
game.phaseLoop(event.beginner);
```

阶段顺序（归档上游文档 `docs/game-startup-flow.md`）：
`phaseZhunbei / phaseJudge / phaseDraw / phaseUse / phaseDiscard / phaseJieshu`
→ **新回合从「准备阶段」开始**。

## 2 逐条判定（对照 `2026-10-18-nc08-cross-line` 的 4 处差异）

| # | 登记差异（旧线 vs 原生） | 上游判定 | 结论 |
|---|---|---|---|
| 1 | 初始阶段/当前座位：旧 `Draw`/座位 1 vs 原生 `Judge`/座位 0 | `beginner` 取 `game.zhu`（主公）→ 首回合为主公；原生主公=座位 0 → **原生一致**；**但原生缺 Prepare 阶段** | **原生与上游一致（座位）**；**Prepare 缺失登记** |
| 2 | 初始手牌：旧 `[6,4,4,4]` vs 原生 `[4,4,4,4]` | `gameDraw` 回调**返回 4**（含主公；仅 `purple` 模式且座位 >5 才 5） | **原生与上游一致**；**旧线偏差** |
| 3 | 一次推进后：旧 `Play`/座位 1 vs 原生 `Draw`/座位 0 | 阶段顺序 `Prepare→Judge→Draw→…` | 原生**无 Prepare** → 相位推进序列整体前移一格（**登记差异**，需按准备阶段类技能逐条核对） |
| 4 | 旧线开局随机伤害（seed 20261602 座位 2 初始 hp=3） | 上游 `identity.js` 该段**无**开局伤害 | **旧线偏差**（或未建模的模式分支）；原生无此行为 |

## 3 回归（新增 `crates/engine-native/tests/nc08_upstream_identity_start.rs`，3 项）

| 判据 | 结果 |
|---|---|
| 主公起手（座位 0）、主公 +1 上限（5）、**每人 4 张**、第 1 回合 | 通过 |
| 原生阶段集合**不含 Prepare**（差异固定，防静默改变）＋起手阶段 `Judge` | 通过 |
| 3 个种子下开局均「每人 4 张」且推进一个阶段仍被接受（状态自洽） | 通过 |

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游判定定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1164 passed / 0 failed / 2 ignored**（1161 → +3） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8265 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办

- [ ] **Prepare（准备阶段）缺失的语义影响**：准备阶段类技能（`phaseZhunbei` 触发）在原生
      的落点（判定/回合起点）需逐条核对；如确需补齐，属规则变化（版本递增 + golden 重录）
- [ ] 旧线偏差（初始手牌 6/开局随机伤害）是否需要同步修正旧线，或明确旧线不再作为对照基准
- [ ] 逐命令对照工具（固定上游版本/局面/随机输入与玩家选择）
- [ ] 更多来源样本（当前 2 来源 + 本开局判定）
