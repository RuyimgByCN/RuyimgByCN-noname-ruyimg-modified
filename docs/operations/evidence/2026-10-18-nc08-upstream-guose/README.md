# NC08 第⑦层样本 31：`guose`（国色，**标准面** / 大乔）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 **2 项差异登记**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-guose.js.txt](upstream-guose.js.txt)）

```js
guose: {
    filter(event, player) { return player.hasCards("hes", { suit: "diamond" }); },
    enable: "chooseToUse",
    filterCard(card) { return get.suit(card) === "diamond"; },
    position: "hes",
    viewAs: { name: "lebu" },
    prompt: "将一张方片牌当乐不思蜀使用",
},
```

## 2 验收结果（`nc08_upstream_guose.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `get.suit === "diamond"` + `viewAs: lebu` | 方块【闪】当【乐不思蜀】使用 → 被接受且**目标判定区 +1** | 通过 |
| 1 | 同上（反例） | **非方块**（红桃【闪】）→ 被拒 | 通过 |
| 1 | 技能门槛 | **未持国色** → 方块牌不可当【乐不思蜀】 | 通过 |
| 2+3 | `position: "hes"` + `viewAs` 选择 | **登记差异①②**：牌区含装备/判定区（原生无该入口）；原生**固定优先级**（国色 > 武圣，源码注释自述） | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:118–132`（`kind != Le && kind != Sha` 前提下，
持国色且牌为**方块** → `kind = CardKind::Le`；随后才轮到武圣）。

## 3 夹具说明

方块牌经**共享助手** `nc08_common::put_hand`（**按 card id**）布阵——本样本按 `Suit::Diamond`
取 id，**不经** `debug_place` 的名字查找（避免同名多花色实例问题）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（`hes` 装备/判定区入口）与制衡/奇袭同族，宜一并评估
- [ ] 差异②（`viewAs` 固定优先级）口径判定
- [ ] 样本 32+：其余来源（当前 **21/37 来源**；标准面技能 **13 个样本**）
