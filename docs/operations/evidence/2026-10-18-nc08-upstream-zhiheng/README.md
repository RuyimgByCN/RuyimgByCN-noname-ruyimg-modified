# NC08 第⑦层样本 25：`zhiheng`（制衡，**标准面** / 孙权）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记，属**规则损失**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-zhiheng.js.txt](upstream-zhiheng.js.txt)）

```js
zhiheng: {
    enable: "phaseUse",
    usable: 1,                       // 每回合限一次
    position: "he",                  // 手牌 + 装备区
    filterCard: true,
    selectCard: [1, Infinity],       // 任意张（≥1）
    allowChooseAll: true,
    prompt: "弃置任意张牌并摸等量的牌",
    async content(event, trigger, player) { await player.draw(event.cards.length); },
},
```

## 2 验收结果（`nc08_upstream_zhiheng.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3+4 | `phaseUse` + `[1,∞]` + `draw(len)` | 出牌阶段发动 → 选 2 张 → 手牌数**不变**（弃 2 摸 2） | 通过 |
| 1 | `usable: 1` | 同回合**第二次发动被拒** | 通过 |
| 2 | `position: "he"` | **差异登记①（规则损失）**：原生候选**只取手牌**（`hand.iter()`）→ 装备区牌**不可**用于制衡 | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/zhi_heng.rs`（`activate` 要求**手牌非空**；
`Select` 阶段选项**仅来自 `player.hand`**；`Draw` 阶段弃置并摸等量）。

## 3 登记的差异②（口径）

上游 `selectCard: [1, Infinity]`（**至少 1**）且不可取消；原生 `min: 0` + `allow_cancel: true`
（**空应答 = 放弃发动**）→ 结果语义等价，但**询问下限/取消口径不同** → 登记，不写成对齐。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异①修复**：候选应含**装备区**牌（`position: "he"`）——`zhi_heng.rs` **未跟踪** → 并入待实施修复清单
- [ ] `allowChooseAll`（一键全选）界面便捷项
- [ ] 样本 26+：其余来源（当前 **21/37 来源**；标准面技能 7 个样本）
