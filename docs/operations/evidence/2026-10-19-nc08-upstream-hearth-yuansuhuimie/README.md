# NC08 第⑦层样本 116：**来源 U07（hearth）** 锦囊 `yuansuhuimie`（元素毁灭）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 三档并列）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`yuansuhuimie`）：

```js
yuansuhuimie: {
    type: "trick", cardnature: "thunder", enable: true,
    selectTarget: -1, filterTarget: true, reverseOrder: true,     // 全体、逆序结算
    content() {
        "step 0"; target.chooseToDiscard([1, 2], "he")…;          // 弃 1–2 张
        "step 1"; if (!result.bool || result.cards.length < 2) { … }   // 少弃 → 雷伤
    },
},
```

依据（规则基准 golden 表第 55 行）：「元素毁灭（**逐目标弃 1–2 每少弃一张受 1 雷伤**）」。

## 2 验收结果（`nc08_upstream_hearth_yuansuhuimie.rs`，三档并列）

| 目标 | 弃牌 | 实测掉血 | 结果 |
|---|---|---|---|
| 1 号 | **弃满 2** | **0** | 通过 |
| 2 号 | **弃 1** | **1** | 通过 |
| 3 号 | **无牌可弃**（少 2） | **2** | 通过 |

**实测**：`询问=["hh_yuansu_discard@obs1","hh_yuansu_discard@obs2"]`（`min 1 / max 2`）；
`1 号 0 / 2 号 1 / 3 号 2`

**原生实现引用**：`content/host/step/hearth.rs`（`HearthYuansuhuimieCard`）。

## 3 夹具要点（**新增一条口径**）

该卡为 `selectTarget: -1`（全体）但**按「无显式目标」受理**（`targets: vec![]`）；
传 `vec![1,2,3]` 会 **`Rejected(IllegalTargets)`** ⇒ **AOE 类卡的目标形态有两种**，
**须两种都试**（不可只按「全体显式列表」一条口径）。

## 4 未完成 / 待办

- [ ] **逆序结算**（`reverseOrder`）的顺序面
- [ ] `cardnature: thunder` 的**属性修饰面**（藤甲等载体）
- [ ] hearth 其余卡（星界沟通/传送门/坦蛇至人）
