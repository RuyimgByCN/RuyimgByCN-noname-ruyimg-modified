# NC08 第⑦层样本 63：`xinfu_jiyuan`（**急援**，标准面 / 伊籍）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（源码事实级）+ **1 项规则损失**（原生**已自述**：仅濒死臂）
- **基准面**：`character/standard/skill.js`
- **命名提示**：`xinfu_jiyuan` → 「**急援**」（`translate.js`）；原生常量 `JI_YUAN`（`names.rs:51`）

## 1 上游源码（归档快照，逐字引用；见 [upstream-jiyuan.js.txt](upstream-jiyuan.js.txt)）

```js
xinfu_jiyuan: {
    trigger: { global: ["dying", "gainAfter", "loseAsyncAfter"] },   // 三个事件臂
    filter(event, player, triggername, target) {
        if (!target?.isIn()) return false;
        if (event.name === "dying") return true;                      // ① 濒死臂
        if (event.giver !== player) return false;                     // ② 给予臂
        if (event.name === "gain") return event.player != player && event.getg(target).length > 0;
        return game.hasPlayer(c => c !== player && event.getg(c).length > 0);   // ③ 失去臂
    },
    async content(event, trigger, player) { await event.targets[0].draw(); },   // 目标摸一张
},
```

## 2 验收结果（`nc08_upstream_jiyuan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `await event.targets[0].draw()` | 原生注释「一名角色进入濒死状态时，可令其**摸一张牌**」——与上游**共用效果**一致 | 通过 |
| 1 | `["dying","gainAfter","loseAsyncAfter"]` | **差异（规则损失，原生已自述）**：原生 `filter` **仅匹配 `EventData::Dying`** → **缺给予臂与失去臂**（**3 臂 → 1 臂**） | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/ji_yuan.rs`（`filter` 仅 `Dying`；头注释「**登记差异：仅濒死臂**」）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] **补两臂**（`gainAfter` 给予臂 + `loseAsyncAfter` 失去臂）——应并入待实施修复清单
- [ ] **濒死臂的行为面**（需构造「角色进入濒死 + 伊籍存活」场景并驱动结算）
- [ ] `audio` 表现面
- [ ] 样本 64+：其余来源/技能（标准面技能 **41 个样本**；来源覆盖 **23/37**）
