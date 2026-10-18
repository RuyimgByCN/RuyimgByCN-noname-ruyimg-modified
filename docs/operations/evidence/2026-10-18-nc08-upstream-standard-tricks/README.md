# NC08 第⑦层样本 4：标准锦囊 `wuzhong` / `taoyuan`（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**上游基准面**（非扩展来源）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**语义点全部对齐**；登记 **2 处差异**（结算顺序 / 目标面接口）

## 1 上游源码（归档快照，逐字引用；见 [upstream-standard-tricks.js.txt](upstream-standard-tricks.js.txt)）

`archive/upstream/noname/apps/core/card/standard.js`：

```js
wuzhong: {
    type: "trick", enable: true, selectTarget: -1, cardcolor: "red",
    toself: true,
    filterTarget(card, player, target) { return target === player; },
    modTarget: true,
    async content(event) { /* versus 分支略 */ await event.target.draw(2); },
},
taoyuan: {
    type: "trick", enable: true, selectTarget: -1, cardcolor: "red",
    reverseOrder: true,
    filterTarget(card, player, target) { return true; },
    ignoreTarget(card, player, target) { return target.isHealthy(); },
    async content(event) { event.target.recover(); },
},
```

## 2 由 JS 推导的规则点 → 原生验收（新增 `nc08_upstream_standard_tricks.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterTarget: target === player` + `draw(2)` | 无中生有**指定他人被拒**；自己结算**摸 2 张** | 通过 |
| 2 | `selectTarget: -1` 且**无** `toself` | 桃园结义**不含使用者**（使用者受伤也不回复） | 通过 |
| 3 | `ignoreTarget: isHealthy()` | **满血者被跳过**（只有受伤者回复 1） | 通过 |
| — | — | **差异 A（登记）**：上游 `reverseOrder: true`（逆序结算）vs 原生按**座位序**遍历 | **已登记** |
| — | — | **差异 B（登记）**：上游把无中生有建模为「自身为目标」（`toself`），原生建模为**无目标锦囊**（`targets` 必须为空，使用者隐式） | **已登记** |

**差异影响分析**：
- 差异 A：`recover` 的**结算顺序**在「回复触发类技能」（如 U07 `nsshuaiyan`「其他角色于你的回合外
  回复体力后」）下可能产生可观测差异 → 是否对齐需按上游语义判定（NC08 待办）；
- 差异 B：语义等价（都只影响自己），但**目标面接口**不同——凡按「目标集合」观察的消费者
  （录像/AI/投影）需注意。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1172 passed / 0 failed / 2 ignored**（1169 → +3） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8276 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC08 第⑦层）

- [ ] 差异 A（桃园结算顺序）的上游判定与（若需）对齐
- [ ] 差异 B（无中生有目标面）是否统一为「自身为目标」建模
- [ ] 样本 5+：其余来源（当前 **3/37 来源**：L20 / U07 / 标准面）
- [ ] 逐命令对照工具；样本 1 顺序差异（`loseHp`/`draw`）上游判定
