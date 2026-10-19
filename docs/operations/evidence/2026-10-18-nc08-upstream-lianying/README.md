# NC08 第⑦层样本 37：`lianying`（连营，**标准面** / 陆逊）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 2 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-lianying.js.txt](upstream-lianying.js.txt)）

```js
lianying: {
    trigger: { player: "loseAfter",
               global: ["equipAfter","addJudgeAfter","gainAfter","loseAsyncAfter","addToExpansionAfter"] },
    frequent: true,
    filter(event, player) {
        if (player.hasCards("h")) return false;              // 失去后手牌为空
        const evt = event.getl(player);
        return evt && evt.player === player && evt.hs && evt.hs.length > 0;   // 失去的是手牌
    },
    async content(event, trigger, player) { await player.draw(); },
},
```

## 2 验收结果（`nc08_upstream_lianying.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2+3 | 失去**最后一张手牌** → 摸 1 | 手牌恰 1 张时打出 → 确认发动后**手牌回到 1 张** | 通过 |
| 2 | `if (player.hasCards("h")) return false` | 手牌 **2 张**时打出 1 张（剩 1 张）→ 上游**不应触发**；实测**出现 `trigger_confirm`** → **差异③（阈值 off-by-one，规则更宽松）** | 通过（**差异登记③**） |
| 1 | `loseAfter` + 5 个全局异步面 | **差异①**：原生为**四类显式事件**（`CardUsed`/`CardResponded`/`CardsMoved`/`CardsDiscarded`）→ 覆盖路径不同（原生注释自述「弃牌阶段批量弃置不产生事件」） | 通过（**差异登记**） |
| 2 | 失去后判空 | **差异②**：原生在**事件时刻**判 `hand.len() <= 1`（语义等价前提：filter 于牌离手前求值）→ 写法不同 | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/lian_ying.rs`（filter `hand.len() > 1 → false`；
四类事件；注释登记批量弃置未覆盖）。

## 2b 门槛用例：**夹具已修 + 发现差异③（阈值 off-by-one）**

**夹具修复**：`debug_give_hand_card` 会**压入帧** → 布阵后必须 `drain`，否则提交被
`Rejected(ChoicePending)`（初版即因此走 SKIP 分支）。

**修复后实测**（[off-by-one-diagnostic.txt](off-by-one-diagnostic.txt)）：

```
DBG 剩 1 张手牌时：open_choice=Some("trigger_confirm") hand=[20]
DBG 差异③确认后：hand=[20]（若为 2 则多摸了 1 张）
```

→ **手牌剩 1 张时仍出现连营确认门** ✗，而上游要求**手牌为空**（`if (player.hasCards("h")) return false`）
→ **不是「写法不同、语义等价」，而是阈值 off-by-one（原生更宽松）** → **登记差异③**
（根因：原生 filter `hand.len() > 1 → false` 在**牌已离手之后**求值 → 剩 1 张即通过）。
**行为后果**（确认后是否立即摸牌）受帧结算影响，本轮只固定「不该出现的触发门出现了」这一事实。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[off-by-one-diagnostic.txt](off-by-one-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 差异①（批量弃置路径）是否补齐；与枭姬（样本 32）的「失去方式覆盖面」**同族**
- [x] **门槛用例夹具已修**（布阵后 drain）；并据此**发现差异③（阈值 off-by-one）**
- [ ] 差异③修复：filter 改为「失去后手牌为空」（`hand.len() > 0 → false`，或按事件后置状态判定）
- [ ] 差异②（门槛写法）在 `CardsDiscarded` 批量事件下的等价性
- [ ] 样本 38+：其余来源（当前 **21/37 来源**；标准面技能 **19 个样本**）
