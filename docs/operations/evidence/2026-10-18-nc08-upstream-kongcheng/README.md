# NC08 第⑦层样本 47：`kongcheng`（空城，**标准面** / 诸葛亮）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-kongcheng.js.txt](upstream-kongcheng.js.txt)）

```js
kongcheng: {
    mod: {
        targetEnabled(card, player, target, now) {
            if (!target.hasCards("h")) {
                if (card.name === "sha" || card.name === "juedou") return false;
            }
        },
    },
    group: "kongcheng1",
    ai: { noh: true, … },
},
```

## 2 验收结果（`nc08_upstream_kongcheng.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `!target.hasCards("h")` + `sha` | 持空城且**无手牌** → 被【杀】指定时 **`Rejected(IllegalTargets)`** | 通过 |
| 1 | 同上（**反例**） | 持空城但**有手牌** → 可被指定（进入 `respond_shan` 询问） | 通过 |
| 1 | `mod.targetEnabled` | **差异登记**：上游为 **`mod` 统一拦截**；原生为**3 处显式校验**（决斗/【杀】/借刀类）→ 语义等价、**新增目标路径时需同步** | 通过（**差异登记**） |

**原生实现引用（3 处）**：`engine/play_card.rs:643`（决斗目标）、`engine/play_sha.rs:306`（【杀】目标）、
`snap/asks.rs:187`（借刀类目标选择）；判据一致：`kongcheng && hand.is_empty() → IllegalTargets`。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `kongcheng1`（`loseEnd` 组技）与 `ai.noh` 表现/策略面
- [ ] 差异（`mod` vs 3 处显式校验）的**维护性风险**：新增「杀/决斗目标路径」时须同步加校验
- [ ] 样本 48+：其余来源/技能（标准面技能 **26 个样本**；来源覆盖 **23/37**）
