# NC08 第⑦层样本 134：**来源 U09（mtg）** 地牌 `mtg_shamolvzhou`（沙漠绿洲）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_shamolvzhou`）：

```js
mtg_shamolvzhou: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        game.changeLand("mtg_shamolvzhou", player);
        player.discoverCard(get.inpile("basic"));    // 从基本牌池「发现」一张并获得
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_shamolvzhou.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **发现（选择）询问** | **`mtg_shamolvzhou_discover`**（候选 **3 张**：`Card(26)/Card(33)/Card(8)`，`min 1/max 1`，不可取消） | 通过 |
| 2 | **获得一张基本牌** | 手牌 **1 → 1**（打出 −1、获得 1）；**手中基本牌 1 张** | 通过 |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Shamolvzhou` → mtg 地牌机）。

## 3 未完成 / 待办

- [ ] `discoverCard` 的**候选集合构成**（上游为 `get.inpile("basic")`；实测候选 3 张）
- [ ] `land_used`（每回合限一次）的重置时机
- [ ] mtg 其余 6 张地牌
