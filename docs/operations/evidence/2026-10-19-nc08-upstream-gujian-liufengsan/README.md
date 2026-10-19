# NC08 第⑦层样本 98：**来源 U05（古剑）** 锦囊 `liufengsan`（六风散）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）** + **1 项差异登记**（**生成 vs 实体取牌**族第 3 例）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`liufengsan`）：

```js
liufengsan: {
    type: "trick", enable: true, fullskin: true, filterTarget: true,
    content() {
        var list = [];
        for (var i = 0; i < 2; i++) list.push(game.createCard("shan"));   // 凭空造 2 张【闪】
        target.gain(list, "gain2");                                      // 目标获得之
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_liufengsan.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 造 2 张【闪】并令目标获得 | **行为面**：1 号手牌 **0 → 2**，且**两张均为【闪】** | 通过 |
| 2 | `game.createCard("shan")`（**生成**） | **差异登记**：**牌堆 146 → 144（减少 2）** ⇒ 原生从**牌堆取实体【闪】**，非生成 → 「**生成 vs 实体取牌**」族**第 3 例** | 登记 |

**原生实现引用**：宿主行为（古剑锦囊机）。

## 3 差异族第 3 例（**本样本证据最直接**）

| 例 | 上游 | 原生 | 观测方式 |
|---|---|---|---|
| `mujiaren`（木甲人） | `game.createCard` 造机关牌 | 牌堆/弃堆取实体牌 | 源码 + 行为面 |
| `yangpijuan`（羊皮卷） | 造牌（候选不属牌堆） | 牌堆发现（候选留在牌堆） | 源码 + 行为面 |
| **`liufengsan`（六风散）** | **造 2 张【闪】** | **牌堆取实体（牌堆 −2）** | **牌堆计数（最直接）** |

⇒ 三例同族；本样本以**牌堆减少量**给出**量化**证据。

## 4 未完成 / 待办

- [ ] 「生成 vs 实体取牌」族的统一口径与修复
- [ ] `ai.result` 策略面
- [ ] 古剑包其余卡牌
