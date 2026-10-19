# NC08 第⑦层样本 132：**来源 U09（mtg）** 地牌 `mtg_bingheyaosai`（冰河要塞）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_bingheyaosai`）：

```js
mtg_bingheyaosai: {
    type: "land", fullborder: "wood",
    enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        "step 0"; game.changeLand("mtg_bingheyaosai", player);
                  player.draw(2);                             // 摸 2
        "step 1"; player.chooseToDiscard("he", 2, true);      // 弃 2（强制）
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_bingheyaosai.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **摸 2** | 牌堆 **114 → 112** | 通过 |
| 2 | **弃 2（强制）** | 询问 **`mtg_bingheyaosai_discard`**（**`min 2 / max 2`**，`allow_cancel=false`） | 通过 |
| 2 | 净效果 | 手牌 **3 → 2**（打出 −1、摸 2、弃 2 ⇒ **净 −1**） | 通过 |

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Bingheyaosai` → mtg 地牌机）。

## 3 未完成 / 待办

- [ ] `land_used`（每回合限一次）的**重置时机**
- [ ] `game.changeLand`（地牌变更）的完整语义
- [ ] mtg 其余 8 张地牌
