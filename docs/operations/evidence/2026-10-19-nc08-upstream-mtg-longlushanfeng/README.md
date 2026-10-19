# NC08 第⑦层样本 137：**来源 U09（mtg）** 地牌 `mtg_longlushanfeng`（龙颅山峰）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（二选一）** + **1 项差异登记**（**「生成 vs 实体取牌」族第 4 例**）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_longlushanfeng`）：

```js
mtg_longlushanfeng: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        "step 0"; game.changeLand("mtg_longlushanfeng", player);
                  player.chooseControl("获得杀", "获得闪").set("prompt", "选择一项");   // 二选一
        "step 1"; if (result.control === "获得杀") {
                      player.gain(game.createCard("sha"), "gain2");                  // **生成【杀】**
                  } else { player.gain(game.createCard("shan"), "gain2"); }          // **生成【闪】**
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_longlushanfeng.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **二选一**（询问 `mtg_longshanfeng_gain`） | 选「获得杀」→ 手中**有【杀】**；选「获得闪」→ 手中**有【闪】**；两者手牌均 **+0**（打出 −1、获得 1） | 通过 |
| 3 | **`game.createCard`（生成）** | **两种选择下牌堆均 −1** ⇒ 原生**从牌堆取实体牌** | ✗ **差异（族第 4 例）** |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Longlushanfeng` → mtg 地牌机）。

## 3 差异归类：「生成 vs 实体取牌」**第 4 例**

| 例 | 上游 | 原生 | 观测 |
|---|---|---|---|
| `mujiaren`（木甲人） | `game.createCard` | 牌堆/弃堆取实体 | 源码+行为 |
| `yangpijuan`（羊皮卷） | 造牌（候选不属牌堆） | 牌堆发现（候选留堆） | 源码+行为 |
| `liufengsan`（流风散） | 造 2 张【闪】 | 牌堆取实体（**−2**） | 牌堆计数 |
| **`longlushanfeng`（龙颅山峰）** | **造【杀】/【闪】** | **牌堆取实体（−1）** | **牌堆计数** |

## 4 未完成 / 待办

- [ ] **生成语义**修复（族内统一口径）
- [ ] `land_used` 每回合重置；mtg 其余 3 张地牌
