# NC08 第⑦层样本 57：`rende`（仁德，**标准面** / 刘备）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（**累计语义完全对齐**；仅 1 项**文档不一致**登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-rende.js.txt](upstream-rende.js.txt)）

```js
rende: {
    enable: "phaseUse",                        // 出牌阶段（无次数限制）
    filterCard: true, selectCard: [1, Infinity], allowChooseAll: true,
    discard: false, lose: false,               // 给出（不是弃置）
    filterTarget(card, player, target) { return player != target; },
    async content(event, trigger, player) {
        const num = player.iterHistory("lose", …).map(evt => evt.cards.length).reduce((a,b)=>a+b,0);
        await player.give(event.cards, event.target);
        if (num < 2 && num + event.cards.length > 1) { await player.recover(); }   // 跨过 2 张
    },
},
```

## 2 验收结果（`nc08_upstream_rende.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | 一次给 2 张 | 自己 hp 3 → **4**（回复 1）；目标手牌 **2 张** | 通过 |
| 3 | **累计** `num < 2 && num + n > 1` | **1+1+1 分三次**：hp `2 → 2 → 3 → 3` —— ①累计 1 张**不回复** ②**跨过 2 张回复 1** ③已跨过**不再回复** | 通过（**逐点对齐**） |
| 1 | **无 `usable`** | 三次发动**均被接受**（可多次发动） | 通过 |
| — | — | **登记（文档不一致，非行为缺陷）**：`rende.rs` 文件头注释写「出牌阶段**限一次**」，与注册面（`limits: SkillLimits::default()` + 注释「可多次发动…累计跨过 2 张回复 1 点」）不一致 → **注释滞后** | 登记 |

**原生实现引用**：`content/standard/skills/rende.rs`（`recover(owner, 1, None)`）；
注册面 `standard_skills.rs:193`。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **修注释**：`rende.rs` 头注释「限一次」→「可多次 + 累计跨过 2 张回复 1 点」（**未跟踪文件**，随拆分落定后改）
- [ ] `check`（AI 给牌评分）与 `audio` 表现/策略面
- [ ] 样本 58+：其余来源/技能（标准面技能 **35 个样本**；来源覆盖 **23/37**）
