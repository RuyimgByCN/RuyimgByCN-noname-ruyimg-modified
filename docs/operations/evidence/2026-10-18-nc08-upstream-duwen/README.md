# NC08 第⑦层样本 17：`duwen`（毒吻，U05 ow 武将包 / 黑白合）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项验收通过**（1 项为差异登记）；**效果面走真实伤害路径，全部通过**

## 1 上游源码（归档快照，逐字引用；见 [upstream-duwen.js.txt](upstream-duwen.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`：

```js
duwen: {
    trigger: { source: "damageBegin" },
    check(event, player) { … },              // AI 策略面
    forced: true,                            // 锁定技
    filter(event, player) {
        return player.countCards("h") === event.player.countCards("h")
            && event.notLink();              // 铁索传导事件**不**加成
    },
    content() { trigger.num++; },            // 伤害 +1
    ai: { … },
},
```

## 2 验收结果（`nc08_upstream_duwen.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2+3 | `forced` + 手牌数相等 + `num++` | 手牌相等 → 1 点伤害实际打出 **2 点**；且**无确认门**（锁定技） | 通过 |
| 2 | `countCards("h")` 相等 | 手牌**不等**（1 vs 3）→ 伤害**不加成** | 通过 |
| 1 | 技能门槛 | **未持技**者手牌相等也**不加成** | 通过 |
| — | `event.notLink()` | 原生按「**铁索传导为独立事件照常判定**」实现 → **差异登记**（源码事实固定） | 通过（**差异登记**） |

**原生实现引用**：`kernel/damage.rs:227`（`hand.len()` 相等 → `amount += 1`）、
`content/host/step/wudian_status.rs:201`（宿主无机器步骤，效果在伤害提交处）。

## 3 工程发现

- 源码事实断言若**整句跨行**（注释折行）会匹配失败 → 应**按行片段**分别断言（本样本首次即踩）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 样本 18+：其余来源（当前 **16/37 来源**）
- [ ] ow 其余 22 名武将的上游样本（本样本为 2/24）
- [ ] `event.notLink()` 差异的最终口径判定（铁索传导是否应排除加成）
- [ ] `check`/`ai` 策略面不在规则对照范围
