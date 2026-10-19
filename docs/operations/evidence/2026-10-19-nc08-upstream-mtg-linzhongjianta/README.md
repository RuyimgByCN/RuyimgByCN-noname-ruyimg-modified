# NC08 第⑦层样本 138：**来源 U09（mtg）** 地牌 `mtg_linzhongjianta`（林中尖塔）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（发现+直接装备）** + **1 项差异登记**（「生成 vs 实体取牌」族**第 5 例**）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_linzhongjianta`）：

```js
mtg_linzhongjianta: {
    type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        "step 0"; game.changeLand("mtg_linzhongjianta", player);
                  player.discoverCard(card => get.subtype(card) === "equip1",
                                      "nogain", "选择一个武器装备之");   // 从武器中选（**不获得**）
        "step 1"; if (result.choice) { player.equip(game.createCa…); }       // **装备之（生成）**
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_linzhongjianta.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **选择武器**（询问 `mtg_linzhongjianta_discover`，候选 3 张） | 已选 = true | 通过 |
| 2 | **直接装备** | 装备区 **0 → 1** | 通过 |
| 2 | **`"nogain"`（不入手牌）** | 手牌 **1 → 0**（仅打出本体） | 通过 |
| 3 | **`game.createCard`（生成）** | **牌堆 −1** ⇒ 原生**取实体牌** | ✗ **差异（族第 5 例）** |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Linzhongjianta` → mtg 地牌机）。

## 3 差异归类：「生成 vs 实体取牌」**第 5 例**

`mujiaren` / `yangpijuan` / `liufengsan`（−2）/ `longlushanfeng`（−1）/ **`linzhongjianta`（−1）**。

## 4 未完成 / 待办

- [ ] **生成语义**修复（族内统一口径）
- [ ] 候选集合是否**限定 `equip1`**；`land_used` 每回合重置
- [ ] mtg 其余 2 张地牌（幽林/依夏兰）
