# NC08 第⑦层样本 12：`linghunzhihuo`（灵魂之火，U05 炉石卡牌子包）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个炉石卡牌子包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**（1 项为差异登记）

## 1 上游源码（归档快照，逐字引用；见 [upstream-linghunzhihuo.js.txt](upstream-linghunzhihuo.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`：

```js
linghunzhihuo: {
    fullskin: true,
    type: "trick",
    cardnature: "fire",
    enable: true,
    filterTarget: true,                    // 任意合法目标（含自己）
    content() {
        "step 0"; target.damage("fire");   // 目标受 1 点火焰伤害
        "step 1";
            var hs = player.getCards("h");
            if (hs.length) player.discard(hs.randomGet());  // 使用者随机弃 1 张手牌
    },
    ai: { … },
},
```

## 2 验收结果（`nc08_upstream_linghunzhihuo.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterTarget: true`（单目标） | **空目标表被拒** | 通过 |
| 2+3 | `target.damage("fire")` + `player.discard(...)` | 目标 **−1 体力**；**使用者**手牌 −2（本体离手 + 弃 1）；日志含伤害事件（火属性） | 通过 |
| 4 | `hs.randomGet()` | 原生为**确定性首张**（`hand.first()`）→ **差异登记**（源码事实固定） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/skills_damage.rs:749`（火伤 `DamageNature::Fire` + 使用者弃手牌首张）、
`content/host/constants.rs:212`（`ht:linghunzhihuo`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 样本 13+：其余来源（当前 **11/37 来源**）
- [ ] 炉石其余 16 张卡的上游样本（本样本为 1/17）
- [ ] 「随机弃置 → 确定性首张」这一**系统性差异族**的最终口径（gwent/古剑/炉石/运筹多处同型）
- [ ] 逐命令对照工具；表现/策略面（`fullskin`/`ai`）不在规则对照范围
