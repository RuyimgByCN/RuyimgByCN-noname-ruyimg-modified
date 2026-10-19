# NC08 第⑦层样本 135：**来源 U09（mtg）** 地牌 `mtg_feixu`（废墟）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_feixu`）：

```js
mtg_feixu: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        game.changeLand("mtg_feixu", player);
        player.discoverCard(ui.discardPile.childNodes);    // 从**弃牌堆**「发现」一张并获得
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_feixu.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **从弃牌堆发现（选择）** | **`mtg_feixu_discover`**（候选 **3 张**：`Card(95)/Card(127)/Card(6)`，`min 1/max 1`） | 通过 |
| 2 | **取走一张** | 弃堆 **19 → 18** | 通过 |
| 2 | 获得 | 手牌 **1 → 1**（打出 −1、获得 1） | 通过 |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Feixu` → mtg 地牌机）。

## 3 夹具要点

先打出一张【杀】使其结算后**入弃牌堆**（实测弃堆 19），再使用废墟 —— 否则弃堆为空、
「从弃牌堆发现」**无候选**，用例会退化为无效观测。

## 4 未完成 / 待办

- [ ] **弃牌堆为空**时的行为（上游 `discoverCard` 无候选）
- [ ] `land_used` 每回合重置；mtg 其余 5 张地牌
