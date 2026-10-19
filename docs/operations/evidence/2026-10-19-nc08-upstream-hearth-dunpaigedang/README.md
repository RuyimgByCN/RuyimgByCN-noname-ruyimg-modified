# NC08 第⑦层样本 114：**来源 U07（hearth）** 锦囊 `dunpaigedang`（盾牌格挡）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**（并**行为面确认**基准既有登记差异）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`dunpaigedang`）：

```js
dunpaigedang: {
    enable: true, type: "trick", toself: true,
    filterTarget(card, player, target) { return player === target; },
    selectTarget: -1, modTarget: true,
    content() {
        "step 0"; target.changeHujia(); target.draw();      // +1 护甲 + 摸 1
        "step 1"; if (target.countCards("he")) target.chooseToDiscard("he", true);   // 弃 1（强制）
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_dunpaigedang.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **+1 护甲 + 摸 1 + 弃 1**（净手牌 −1） | 护甲 **0 → 1**；手牌 **3 → 2**（打出 −1、摸 1、弃 1） | 通过 |
| — | **登记差异的行为面确认** | **`询问=[]`** ⇒ 弃 1 为**自动弃置（确定性首张）**，与基准 golden 表第 55 行「**确定性首张**」一致 | 确认 |

**原生实现引用**：`kernel/intrinsic.rs:440/449`（`CardKind::Dunpaigedang`）。

## 3 说明（**先验「登记差异」，避免把差异当缺陷**）

初版断言「应出现弃 1 的询问」→ 失败；核对**基准既有登记**后确认：上游为**玩家选择弃哪张**，
原生为**自动弃置** ⇒ **无询问是已登记差异**，**不是缺陷**。本样本据此改为「净效果 + 差异确认」。

## 4 未完成 / 待办

- [ ] `modTarget` 的额外影响面
- [ ] **自动弃置具体弃了哪张**（本样本只验净效果）
- [ ] hearth 其余卡（治疗波/元素毁灭/星界沟通/传送门/坦蛇至人）
