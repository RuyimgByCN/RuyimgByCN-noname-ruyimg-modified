# NC08 第⑦层样本 43：`shushen`（淑慎，**标准面** / 甘夫人）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记，属**规则损失**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-shushen.js.txt](upstream-shushen.js.txt)）

```js
shushen: {
    audio: "shushen",
    trigger: { player: "recoverEnd" },              // 自己回复体力后
    getIndex(event) { return event.num || 1; },     // 按回复点数计次
    async cost(event, trigger, player) {
        event.result = await player.chooseTarget({ filterTarget: lib.filter.notMe }).forResult();
    },
    async content(event, trigger, player) {
        const target = event.targets[0];
        await target.draw(target.hasCards("h") ? 1 : 2);   // 有手牌摸 1、无手牌摸 2
    },
},
```

## 2 验收结果（`nc08_upstream_shushen.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 3 | `hasCards("h") ? 1 : 2`（**无手牌**） | 目标空手牌 → 确认发动并选其后**摸 2**（实测 `0 → 2`） | 通过 |
| 3 | 同上（**有手牌**） | 目标持 1 张 → 摸 **1**（实测 `1 → 2`） | 通过 |
| 1 | `getIndex(event) { return event.num \|\| 1; }` | **差异登记（规则损失）**：上游**按回复点数计次**（回复 N 点触发 N 次）；原生 filter 只判 `amount > 0` 且机器**单次结算** → 回复 2 点只触发 1 次 | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/shu_shen.rs`（filter 判 `Recover` + `amount > 0` + 存活；
`Read` 阶段按 `hand.is_empty()` 取 2/1 后 `draw_cards(target, n)`）。

## 3 夹具说明（写入证据）

- **真实回复路径**：`debug_recover` 只发 `Recover` 事件**不实际回血** → 本样本用**打出【桃】**
  （自我目标牌以 `targets: vec![]` 提交）触发真实回复；
- 行为面在**未出现确认门**时打印 **`NOTE`**（不写成通过）；本次两处均出现（[branch-diagnostic.txt](branch-diagnostic.txt) 中 NOTE 行数为 0）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[branch-diagnostic.txt](branch-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异修复**：按回复点数计次（`amount` 次各自选目标）——`shu_shen.rs` **未跟踪** → 并入待实施修复清单
- [ ] 一次回复 N 点的**行为面**（需构造多点回复）
- [ ] 样本 44+：其余来源/技能（标准面技能 **22 个样本**；来源覆盖 **23/37**）
