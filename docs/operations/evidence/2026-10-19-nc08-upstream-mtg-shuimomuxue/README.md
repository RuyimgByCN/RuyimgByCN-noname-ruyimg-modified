# NC08 第⑦层样本 136：**来源 U09（mtg）** 地牌 `mtg_shuimomuxue`（水没墓穴）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面 · 两支）**，**未发现差异**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`（`mtg_shuimomuxue`）：

```js
mtg_shuimomuxue: {
    type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
    notarget: true,
    content() {
        "step 0"; game.changeLand("mtg_shuimomuxue", player);
                  if (player.countDiscardableCards("he")) {
                      player.chooseToDiscard("是否弃置一张牌并摸两张牌？", "he")…;   // 可取消
                  } else { event.fin… }
    },
},
```

## 2 验收结果（`nc08_upstream_mtg_shuimomuxue.rs`，**两支**）

| 分支 | 实测 | 结果 |
|---|---|---|
| **弃 1 张** | **摸 2**（牌堆 −2）；手牌 **+0**（打出 −1、弃 1、摸 2） | 通过 |
| **取消** | **摸 0**；手牌 **−1**（仅打出本体） | 通过 |

**询问**：`mtg_shuimomuxue_discard`（两支均出现；取消支走 `allow_cancel`）。

**原生实现引用**：`kernel/intrinsic.rs`（`CardKind::Shuimomuxue` → mtg 地牌机）。

## 3 未完成 / 待办

- [ ] **无可弃牌**分支（上游 `countDiscardableCards("he")` 为 0 时直接结束）
- [ ] `land_used` 每回合重置；mtg 其余 4 张地牌
