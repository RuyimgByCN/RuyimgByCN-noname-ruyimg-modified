# NC08 第⑦层样本 45：`fankui`（反馈，**标准面** / 司马懿）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **2 项差异登记**，其中①为「玩家选择权缺失」族第 2 例）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-fankui.js.txt](upstream-fankui.js.txt)）

```js
fankui: {
    trigger: { player: "damageEnd" },
    logTarget: "source",
    filter(event, player) {
        return event.num > 0
            && event.source?.hasGainableCards(player, event.source !== player ? "he" : "e");
    },
    async content(event, trigger, player) {
        player.gainPlayerCard({ target: trigger.source,
            position: trigger.source !== player ? "he" : "e", forced: true });
    },
},
```

## 2 验收结果（`nc08_upstream_fankui.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `damageEnd` + `gainPlayerCard` | 1 号用【杀】打 0 号 → 0 号（持反馈）**获得来源一张牌**（来源手牌 1 → 0） | 通过 |
| 3 | `gainPlayerCard`（**玩家选牌**） | **差异①**：原生取**确定性首张**（`.hand` 优先、否则 `equips.first()`）→ 与 `lijian` 的「**玩家选择权缺失**」**同族（第 2 例）** | 通过（**差异登记**） |
| 2 | `event.source !== player ? "he" : "e"` | **差异②（原生更宽松）**：上游**自伤**时**仅装备区**（`"e"`）；原生先取 `.hand` → 自伤时**仍可拿手牌** | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/fan_kui.rs`（filter `Damage` + 目标自己 + 未防止 +
`amount > 0` + `event_damage_source_stealable`；取牌 `hand`/`equips` 首张）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 差异①（玩家选牌）随「多目标/选牌选择权」族一并评估（当前 2 例：lijian、fankui）
- [ ] 差异②（自伤牌区）口径判定与修复（属**原生更宽松**）
- [ ] 样本 46+：其余来源/技能（标准面技能 **24 个样本**；来源覆盖 **23/37**）
