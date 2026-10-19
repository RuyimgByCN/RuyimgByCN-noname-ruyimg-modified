# NC08 第⑦层样本 58：`jijiang`（激将，**标准面** / 刘备）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（**源码事实级**）+ 1 项**同族设计差异**引用
- **基准面**：`character/standard/skill.js`
- **样本类型**：**源码事实级**（代打询问队列的**行为面**登记待办）

## 1 上游源码（归档快照，逐字引用；见 [upstream-jijiang.js.txt](upstream-jijiang.js.txt)）

```js
jijiang: {
    group: ["jijiang1"], zhuSkill: true,                    // 主公技
    filter(event, player) {
        if (!player.hasZhuSkill("jijiang")
            || !game.hasPlayer(c => c !== player && c.group === "shu")) return false;
        return !event.jijiang && (event.type !== "phase" || !player.hasSkill("jijiang3"));
    },
    enable: ["chooseToUse", "chooseToRespond"],
    viewAs: { name: "sha" },
    filterCard() { return false; },                         // 不用自己的牌
    selectCard: -1,
},
```

## 2 验收结果（`nc08_upstream_jijiang.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `hasZhuSkill` + `group === "shu"` | 原生 `activate` 要求 `is_lord(owner)` + 存在 `Faction::Shu` 角色；注释「**不以暗手牌预筛**」（与上游一致） | 通过 |
| 2 | `filterCard() → false` + `selectCard: -1` | 原生以 **`HelperAsk`（代打）** 形态实现；`activate` **不检查手牌**（对照：青囊/离间要求手牌非空） | 通过 |
| 1 | `hasZhuSkill` | **同族设计差异引用**：原生用 `is_lord`，其在**非身份局返回 true**（`state.rs` 注释「无身份体系的模式持技即可」）→ **原型模式下激将亦可发动**（与 `hujia` 同族，**已核对为有意设计**，见修复清单第 17 项） | 通过（**同族引用**） |

**原生实现引用**：`content/standard/skills/jijiang.rs`（`activate`：`is_lord` + `Faction::Shu`；
机器 `HelperAsk` → …）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **代打询问队列的行为面**（需构造「需要【杀】」场景：如响应【杀】或主动使用）
- [ ] `jijiang3` 相关分支；`audio` 表现面
- [ ] 样本 59+：其余来源/技能（标准面技能 **36 个样本**；来源覆盖 **23/37**）
