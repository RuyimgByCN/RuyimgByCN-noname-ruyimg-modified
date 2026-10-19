# NC08 第⑦层样本 60：`yaowu`（耀武，**标准面** / 华雄）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（正反例 + 1 项**注释滞后观察**）
- **基准面**：`character/standard/skill.js`
- **里程碑**：本样本为 **NC08 第 60 个样本**

## 1 上游源码（归档快照，逐字引用；见 [upstream-yaowu.js.txt](upstream-yaowu.js.txt)）

```js
yaowu: {
    trigger: { player: "damageBegin3" },
    forced: true,
    filter(event, player) {
        return event.card?.name === "sha" && get.color(event.card) === "red" && event.source?.isIn();
    },
    async content(event, trigger, player) {
        await trigger.source.chooseDrawRecover({ forced: true });   // 来源二选：摸 1 或回复 1
    },
},
```

## 2 验收结果（`nc08_upstream_yaowu.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | 红色【杀】+ `chooseDrawRecover` | **红色**【杀】造成伤害 → 出现 **`yaowu_draw_or_recover`** 询问（来源二选） | 通过 |
| 1 | `get.color(card) === "red"`（**反例**） | **黑色**【杀】→ **无任何询问**（`询问=[]`） | 通过 |
| — | `chooseDrawRecover({ forced: true })` | **登记（注释可能滞后，待核对）**：`yao_wu.rs` 头注释称「登记差异：上游由 AI/玩家确认，原生为**强制二选**」，但**上游本身即 `forced: true`** → 该「差异」**疑不成立**（与样本 57 仁德注释滞后**同类**） | 登记 |

**原生实现引用**：`content/standard/skills/yao_wu.rs`（filter：目标自己 + 未防止 + 来源存在/非自己/存活
+ 牌为【杀】且花色**红桃/方块**；机器 `if (p.hp as u32) < p.max_hp` → **满血时无回复选项**）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **来源二选的行为面**（需构造来源为玩家、可观察其手牌/体力变化的场景）
- [ ] 核对「注释所称差异」是否成立（若否，按样本 57 同类处理：改注释）
- [ ] `ai.neg` 策略面
- [ ] 样本 61+：其余来源/技能（标准面技能 **38 个样本**；来源覆盖 **23/37**）
