# NC08 第⑦层样本 91：**来源 U05（古剑）** 机关牌 `feibiao`（飞镖）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 三分支）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`feibiao`）：

```js
feibiao: {
    type: "jiguan", enable: true, wuxieable: true,
    outrange: { globalFrom: 2 },                  // 无视距离（攻击范围 +2）
    filterTarget(card, player, target) { return target !== player; },
    content() {
        "step 0";
        if (!target.countCards("h", { color: "black" })) { target.loseHp(); event.finish(); }
        else target.chooseToDiscard({ color: "black" }, "弃置一张黑色手牌或受失去1点体力")…;
        "step 1";
        if (!result.bool) target.loseHp();        // 未弃 → 失去 1 点体力（体力流失，非伤害）
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_feibiao.rs`，三分支实测）

| 分支 | 实测 | 结果 |
|---|---|---|
| 有黑色手牌且**弃之** | 询问 **`gj_feibiao_discard`** → **hp +0**（不流失） | 通过 |
| **无**黑色手牌 | **无询问**（上游 `event.finish()`）→ **hp −1**（直接流失） | 通过 |
| 有黑色手牌但**未弃** | **hp −1**（流失） | 通过 |

**原生实现引用**：`content/host/step/gujian.rs:90`（`GujianFeibiaoCard` 阶段机）。

## 3 未完成 / 待办

- [ ] **`loseHp` 与 `damage` 的语义差异**（需护甲/伤害修饰载体才能区分——本样本只验证净效果）
- [ ] `outrange`（**无视距离**：`globalFrom = 2`）的射程面
- [ ] `wuxieable` 无懈面
- [ ] 古剑包其余卡牌
