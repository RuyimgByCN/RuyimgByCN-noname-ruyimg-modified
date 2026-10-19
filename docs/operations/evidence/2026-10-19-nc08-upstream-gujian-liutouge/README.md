# NC08 第⑦层样本 101：**来源 U05（古剑）** 机关牌 `liutouge`（六骰格）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含行为面两支）+ **1 项差异登记**（**随机→确定性**族第 3 例）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`liutouge`）：

```js
liutouge: {
    type: "jiguan", enable: true, filterTarget: true, wuxieable: true,
    content() {
        if (player.getEnemies().includes(target)) target.getDebuff();   // 敌方 → 减益
        else target.getBuff();                                          // 否则 → 增益
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_liutouge.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 增益/减益两支 | **行为面**：对自己（增益）手牌**净变化 0**（打出本体 −1、**摸 1**）；对其他角色（减益）**净变化 −1**（**弃首张手牌**） | 通过 |
| 2 | `getBuff()`/`getDebuff()`（**效果池**） | **差异登记**：原生代码注释自述「自己 → 摸 1（**增益简化**）；其他角色 → 弃首张手牌（**减益简化**；**登记差异：上游随机效果池**）」⇒ 「**随机→确定性**」族**第 3 例** | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/jiguan.rs:361`（`HostBehavior::JiguanLiutougeCard`）。

## 3 夹具要点

- `filterTarget: true` → 目标须**显式**给出（**对自己即 `vec![0]`**；`vec![]` 会 `Rejected(IllegalTargets)`）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 上游**效果池的具体条目**（需读 `getBuff`/`getDebuff` 实现）
- [ ] `wuxieable` 无懈面
- [ ] 古剑包其余卡牌（机关族余：`bingpotong` 天女散花）
