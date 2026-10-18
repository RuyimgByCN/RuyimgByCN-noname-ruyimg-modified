# NC08 第⑦层样本 20：`jianxiong`（奸雄，**标准面** / 曹操）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**；其中 **2 项为差异登记**

## 1 上游源码（归档快照，逐字引用；见 [upstream-jianxiong.js.txt](upstream-jianxiong.js.txt)）

`archive/upstream/noname/apps/core/character/sb/skill.js`：

```js
jianxiong: {
    trigger: { player: "damageEnd" },
    frequent: true,
    group: "sbjianxiong_mark",
    filter(event, player) {
        return (get.itemtype(event.cards) === "cards"
                && event.cards.some(i => get.position(i, true) === "o"))   // 伤害牌在**处理区**
            || 3 - player.countMark("sbjianxiong") > 0;                    // 或标记 < 3
    },
    prompt2(event, player) { … },   // 二选一：获得伤害牌 / 摸牌
},
```

## 2 验收结果（`nc08_upstream_jianxiong.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `damageEnd` + 获得伤害牌 | 受到**带牌**伤害后确认发动 → 手牌增加（获得该牌） | 通过 |
| 2 | `get.position(i, true) === "o"`（处理区） | **差异①（位置口径）**：原生门槛为「**牌已在弃牌堆**」（源码事实固定） | 通过（**差异登记①**） |
| 2+3 | `countMark("sbjianxiong")` + `prompt2` | **差异②**：原生**无标记计数**、无「改为摸牌」分支 | 通过（**差异登记②**） |

**原生实现引用**：`content/standard/skills/jian_xiong.rs`（filter：`d.card.is_some()` 且
`discard.contains(card)`；机器：弃牌堆 → 手牌；`OptionalGate` 确认）。

## 3 口径说明（不夸大）

行为面用例在「伤害未带牌/未命中」时**如实打印 SKIP 并返回**（不写成通过）；本次运行
**出现了确认门并验证了获得路径**（见 [targeted.log](targeted.log)）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（处理区 vs 弃牌堆）口径判定
- [ ] 差异②（`sbjianxiong` 标记 + 摸牌面）是否补齐
- [ ] 样本 21+：其余来源（当前 **17/37 来源**；标准面 2 个样本）
