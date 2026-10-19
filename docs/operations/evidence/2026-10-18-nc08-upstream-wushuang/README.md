# NC08 第⑦层样本 50：`wushuang`（无双，**标准面** / 吕布）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`
- **里程碑**：本样本为 **NC08 第 50 个样本**

## 1 上游源码（归档快照，逐字引用；见 [upstream-wushuang.js.txt](upstream-wushuang.js.txt)）

```js
wushuang: { forced: true, locked: true, group: ["wushuang1", "wushuang2"], … },
wushuang1: {                                   // 【杀】方向
    trigger: { player: "useCardToPlayered" }, forced: true, sourceSkill: "wushuang",
    filter(event, player) {
        return event.card.name === "sha"
            && !event.getParent()?.directHit.includes(event.target);   // 必中时排除
    },
    async content(event, trigger, player) { … 需两张【闪】… },
},
// wushuang2：决斗方向（需两张【杀】）
```

## 2 验收结果（`nc08_upstream_wushuang.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `group: ["wushuang1","wushuang2"]` | **两个方向均有实现**：【杀】方向 `sha.rs`（`source_has_wushuang`，注释「需要连续两张闪」）、决斗方向 `duel.rs`（`opponent_has_wushuang`，「同一响应者再次询问第二张杀」） | 通过 |
| 1 | 需**两张【闪】** | 目标只打出**一张【闪】**→ **不足以**抵消，目标受伤（hp 4 → 3） | 通过 |
| 1 | `!directHit.includes(target)` | **差异登记**：原生 `source_has_wushuang` **只判是否持技**、未见 `directHit`（必中）判定 → 行为面待办 | 通过（**差异登记**） |

**原生实现引用**：`kernel/sha.rs:277`（【杀】方向）、`kernel/duel.rs:121`（决斗方向）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed**（NOTE 行数 0） | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **差异（`directHit` 必中排除）**：行为面（需构造「必中」场景）与修复口径
- [ ] `wushuang2` **决斗方向的行为面**（本样本只固定源码事实）
- [ ] `preHidden`/`audioname` 表现面
- [ ] 样本 51+：其余来源/技能（标准面技能 **29 个样本**；来源覆盖 **23/37**）
