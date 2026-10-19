# NC08 第⑦层样本 102：**来源 U05（古剑）** 机关牌 `bingpotong`（天女散花）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含行为面两分支 + 语义判别）；**并更正基准一处旧登记**（见 §4）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`bingpotong`）：

```js
bingpotong: {
    type: "jiguan", enable: true, wuxieable: true,
    filterTarget(card, player, target) { return target.countCards("h") > 0; },
    selectTarget: [1, 3],
    content() {
        "step 0"; player.chooseCard("请展示一张手牌", true)…
        "step 1"; target.chooseCard("请展示一张手牌", true)…
                  .prompt2 = "若与…展示的牌相同，…弃置展示的牌，你失去1点体力";
        "step 3"; if (get.color(event.card2) === get.color(event.card1)) { … }
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_bingpotong.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `prompt2`：同色 → 使用者弃展示牌 + 目标**失去 1 点体力** | **行为面（同色）**：使用者手牌 **−2**（打出本体 −1、弃展示牌 −1）、目标手牌 **0**（**展示不移动牌**）、目标 hp **−1** | 通过 |
| 2 | 异色无事 | **行为面（异色）**：使用者 **−1**（仅本体）、目标手牌 **0**、hp **0** | 通过 |
| 2 | 「失去体力」语义 | **语义判别**：目标预置**护甲 2** → hp **−1**、护甲 **+0** ⇒ **`loseHp` 语义**（**不被护甲吸收**，非 `damage`） | 通过 |

**原生实现引用**：`content/host/step/jiguan.rs:8-10`（注释：「使用者自选展示一张 → 目标自选展示一张 →
**同色则使用者弃展示牌、目标失去 1 点体力**（**规则 v77 原语**）；异色无事。展示不移动牌」）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 **更正规则基准一处旧登记**（诚实记录）

规则基准 golden 表（第 51 行，G1 古剑机关簇）对天女散花记为
「**登记差异：确定性首张+失去体力简化为伤害**」——本样本以**护甲载体**实测：
目标**有护甲 2 时仍掉 1 点血、护甲不变** ⇒ 原生用的是 **`loseHp`（失去体力）**，
**并非**「简化为伤害」⇒ 该旧登记**过时/不准确**，已在基准中更正。

**保留的差异**：「**确定性首张**」（上游 `chooseCard` 由玩家/随机选，原生取首张）仍成立。

## 5 未完成 / 待办

- [ ] `selectTarget: [1,3]` 的**多目标**面
- [ ] `wuxieable` 无懈面
- [ ] 古剑包其余卡牌
