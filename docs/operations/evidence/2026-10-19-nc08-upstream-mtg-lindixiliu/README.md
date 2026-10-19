# NC08 第⑦层样本 133：**来源 U09（mtg）** 地牌 `mtg_lindixiliu`（林地溪流）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_lindixiliu`）：

```js
mtg_lindixiliu: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        "step 0"; game.changeLand("mtg_lindixiliu", player);
                  var card = get.cardPile2(card => get.suit(card) === "heart");   // 牌堆找红桃
                  if (card) player.gain(card, "draw");                            // 获得之
        "step 1"; player.chooseToDiscard("he", [1, Infinity], { suit: "heart" }, "林地溪流")…;
                  // 弃任意张红桃（可取消；每弃一张将一张延时锦囊置入随机敌方判定区）
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_lindixiliu.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **从牌堆取一张红桃并获得** | 牌堆 **116 → 115**；手牌 **1 → 1**（打出 −1、获得 1）；**手中红桃 1 张** | 通过 |
| 3 | **第二步可取消** | 询问 **`mtg_lindixiliu_discard`**（**`min 0 / max 1`**、**`allow_cancel=true`**）⇒ 取消后无后续结算 | 通过 |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Lindixiliu` → mtg 地牌机）。

## 3 未完成 / 待办

- [ ] 第二步**每弃一张置延时锦囊**的完整链路
- [ ] **随机敌方**（登记差异面）
- [ ] `land_used` 每回合重置；mtg 其余 7 张地牌
