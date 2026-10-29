# NC08 第⑦层样本 46：`yiji`（遗计，**标准面** / 郭嘉）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 2 项差异登记）
- **基准面**：`character/standard/skill.js`
- **命名提示**：该技能在原生是常量 **`JI_JI`**（`vocabulary/names.rs:106`，注释写明「郭嘉「遗计」」）
  ——按上游键名 `yiji` 检索**查不到**（见[实现索引](../2026-10-18-nc08-source-coverage/standard-skill-implementation.txt)）

## 1 上游源码（归档快照，逐字引用；见 [upstream-yiji.js.txt](upstream-yiji.js.txt)）

```js
yiji: {
    trigger: { player: "damageEnd" },
    frequent: true,
    filter(event) { return event.num > 0; },
    getIndex(event, player, triggername) { return event.num; },   // 按伤害点数计次
    async content(event, trigger, player) {
        const cards = get.cards(2);                                // 亮出牌堆顶 2 张
        await game.cardsGotoOrdering(cards);
        …                                                          // 分配给其他角色
    },
},
```

## 2 验收结果（`nc08_upstream_yiji.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `damageEnd` + 摸 2 | 1 号用【杀】打 0 号（持遗计）→ 确认发动后**手牌 0 → 2** | 通过 |
| 1 | `getIndex → event.num` | **差异①**：上游**按伤害点数计次**；原生只判 `amount > 0`、**单次结算** → 与 `shushen` **同族（第 2 例）** | 通过（**差异登记**） |
| 2 | `get.cards(2)` + `cardsGotoOrdering` | **差异②（机制不同）**：上游**亮出牌堆顶 2 张**再分配**这 2 张**；原生**摸进手牌**（`draw_cards`）后**从手牌中**给出 → 给出的牌可能不是那 2 张 | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/ji_ji.rs`（filter `Damage` + 目标自己 + 未防止 +
`amount > 0`；`Draw` → `draw_cards(owner, 2)` → `Target`/`GiveCard`/`GiveChooseCard` → `give_hand_card`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed**（NOTE 行数 0） | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 差异①（按点数计次）随该族评估（当前 2 例：shushen、yiji）
- [ ] 差异②（亮出 vs 摸入手牌）的**信息面**可见性差异是否影响联机投影
- [ ] 样本 47+：其余来源/技能（标准面技能 **25 个样本**；来源覆盖 **23/37**）
