# NC08 第⑦层样本 88：**来源 U05（古剑）** 锦囊 `shujinsan`（舒筋散）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`shujinsan`）：

```js
shujinsan: {
    type: "trick", enable: true,
    filterTarget(card, player, target) { return target.countCards("he") > 0; },
    content() {
        "step 0";
        target.chooseToDiscard("he", [1, target.countCards("he")], "弃置任意张牌并摸等量的牌");
        "step 1";
        if (result.bool) target.draw(result.cards.length);
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_shujinsan.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `chooseToDiscard("he",[1,n])` + `draw(result.cards.length)` | **行为面**：询问 **`gj_shujinsan_discard`**（`min 1 / max 2`，与上游 `[1, countCards("he")]` 一致）→ 弃 2 张手牌 → **手牌 2 → 2**（弃 2 摸 2）、被弃牌离手、**装备区牌未受影响** | 通过 |
| 1 | `filterTarget`：`countCards("he") > 0` | **门槛反例**：目标 `he` 无牌 → **`Rejected(IllegalTargets)`** | 通过 |

**原生实现引用**：宿主行为（野升样本注释称「**舒筋散机器同构**」）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `ai`（弃牌评分）策略面
- [ ] 古剑包其余卡牌
