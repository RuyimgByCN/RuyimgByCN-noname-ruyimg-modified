# NC08 第⑦层样本 115：**来源 U07（hearth）** 锦囊 `zhiliaobo`（治疗波）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过（含行为面 · 两分支 + 门槛反例）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`zhiliaobo`）：

```js
zhiliaobo: {
    enable: true, type: "trick",
    filterTarget(card, player, target) { return target.hp < target.maxHp; },
    content() {
        "step 0"; target.judge(function (card) { return get.color(card) === "red" ? 1 : 0; });
        "step 1"; if (result.bool) target.recover();   // 红 → 回复 1
                  else target.changeHujia();           // 非红 → +1 护甲
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_zhiliaobo.rs`，2 项）

| # | 分支 | 实测 | 结果 |
|---|---|---|---|
| 2 | **红判定 → 回复 1** | hp **+1**、护甲 **+0** | 通过 |
| 2 | **非红判定 → +1 护甲** | hp **+0**、护甲 **+1** | 通过 |
| 1 | **门槛反例**：满血目标 | **`Rejected(IllegalTargets)`** | 通过 |

**原生实现引用**：`content/host/step/hearth.rs:44`（`HearthZhiliaoboCard`）；
`kernel/intrinsic.rs:441/455`（`CardKind::Zhiliaobo`）。

## 3 夹具要点（**纪律应用**）

判定取**牌堆顶** ⇒ 用 `debug_set_deck_top(名)` 后**回读 `deck.last()` 的牌色**，
循环到顶牌为所需颜色为止（「**置顶牌须回读核对**」纪律的直接应用；本样本一次通过）。

## 4 未完成 / 待办

- [ ] 判定牌的**去向**（判定后入弃堆）与「改判」类技能交互
- [ ] hearth 其余卡（元素毁灭/星界沟通/传送门/坦蛇至人）
