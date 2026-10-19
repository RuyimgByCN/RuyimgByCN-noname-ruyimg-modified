# NC08 第⑦层样本 66：**来源 U05（杀海拾遗·swd）** 机关牌 `mujiaren`（木甲人）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **2 项差异登记**）；**来源 U05 样本深度 +1**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:490`：

```js
mujiaren: {
    enable: true, type: "jiguan", usable: 1, updateUsable: "phaseUse", forceUsable: true,
    wuxieable: true, selectTarget: -1,
    filterTarget(card, player, target) { return target === player; },   // 仅自己
    content() {
        "step 0";
        var cards = target.getCards("h", c => get.type(c) !== "basic");  // 非基本手牌
        if (cards.length) target.lose(cards)._triggered = null;          // 静默失去
        event.num = 1 + cards.length;                                    // 1 + 失去数
        "step 1";
        var list = get.typeCard("jiguan");
        for (var i = 0; i < list.length; i++) if (lib.card[list[i]].derivation) list.splice(i--, 1);
        if (list.length) { … target.directgain(cards); }                 // 随机生成机关牌
    },
},
```

## 2 验收结果（`nc08_upstream_swd_mujiaren.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | 失去非基本手牌 N 张 → 得 **1+N** 机关牌 | **行为面**：0 号持 2 张非基本牌 + 本体 → 发动后**两张非基本牌均不在手**、手牌中**机关牌 3 张**（=1+2）；手牌数 3 → 3（失去 3 张含本体、得 3 张） | 通过 |
| 3 | `game.createCard(list.randomGet())` | **差异①**：上游按**牌定义池生成新牌**；原生从**实体牌堆/弃堆**取已有牌（注释自述「实体发现」）→ 同数量、**不同来源** | 通过（**差异登记**） |
| 3 | `if (lib.card[list[i]].derivation) list.splice(...)` | **差异②**：上游剔除 `derivation` 卡；原生过滤为 `kind == Jiguan && != self.card`，**未见 derivation 排除**（未判定影响） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/swd_card_effects.rs:552`（弃非基本手牌 → `num = 1 + dropped.len()`
→ 牌堆+弃堆 Jiguan 池随机取 num 张）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 差异①（生成 vs 实体）在**牌堆耗尽**时的行为差异（原生注释称「不足给余量」）
- [ ] 差异②（`derivation` 排除）是否影响可选池
- [ ] `wuxieable` 无懈面；`forceUsable`/`updateUsable` UI 面
- [ ] 样本 67+：其余来源/卡牌（**swd 90 卡**仍有大量未采样）
