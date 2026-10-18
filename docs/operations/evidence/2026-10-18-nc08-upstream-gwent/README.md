# NC08 第⑦层样本 10：`gw_yanziyaoshui`（燕子药水，U05 gwent 卡牌子包）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 gwent 卡牌子包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**由上游 JS 推导的 3 条规则点全部对齐**（无新差异登记）

## 1 上游源码（归档快照，逐字引用；见 [upstream-yanziyaoshui.js.txt](upstream-yanziyaoshui.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js`：

```js
gw_yanziyaoshui: {
    fullborder: "bronze",
    type: "spell", subtype: "spell_bronze",
    enable: true,
    filterTarget: true,                 // 任意合法目标（含自己）
    content() {
        if (target.isMinHandcard()) {   // 目标是手牌最少者（之一）
            target.draw(2);
        } else {
            target.draw();
        }
    },
    ai: { … },
},
```

## 2 由 JS 推导的规则点 → 原生验收（`nc08_upstream_yanziyaoshui.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterTarget: true`（单目标法术） | **空目标表**与**多目标**均被拒 | 通过 |
| 2 | `target.isMinHandcard()` → `draw(2)` | 目标手牌 1 张（全场最少）→ **摸 2** | 通过 |
| 3 | 否则 `draw()` | 目标 3 张而他人有 1 张（非最少）→ **摸 1** | 通过 |

**原生实现引用**：`engine/play_card.rs:1508`（恰 1 名存活目标校验）、
`kernel/intrinsic.rs:363`（无懈窗口包装 + `GwentYanzhiCard`）、
`content/host/step/gwent.rs:6`（`alive_players().all(hand.len() >= t.hand.len())` → 2/1）。

## 3 工程发现（写入证据，供后续样本复用）

- gwent 目录牌名带**花色点数后缀**（如 `gw_yanziyaoshui#heart8`）→ `debug_place` 的**精确名匹配**
  会 panic（`unknown card name`）；正确做法是**按前缀从目录取精确名**再 `debug_place`；
- `debug_give_hand_card(from, to, card)` 要求牌**已在 `from` 手牌**（否则 `CardNotInHand`）→
  构造「给某座位一张牌」应先用 `debug_place(player, name)` 置入，再按需转移。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1191 passed / 0 failed / 6 ignored** | — |
| 行数门禁 | 8289 文件 ≤ 2000 行 | — |

## 5 未完成 / 待办

- [ ] 样本 11+：其余来源（当前 **9/37 来源**：L20 / U07 / 标准面 / L19 / L06 / swd / L10 / U05 运筹 / U05 gwent）
- [ ] gwent 其余 20 张卡的上游样本（本样本为 1/21）
- [ ] 逐命令对照工具（需上游 JS harness）
- [ ] 表现/策略面（`fullborder`/`subtype`/`ai`）不在规则对照范围（边界登记）
