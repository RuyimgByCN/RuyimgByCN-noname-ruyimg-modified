# NC08 第⑦层样本 68：**来源 U05（杀海拾遗·swd）** 机关牌 `jiguantong`（机关火筒）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **1 项差异登记**）；**来源 U05 样本深度 +1**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`（`jiguantong`）：

```js
jiguantong: {
    type: "jiguan", cardnature: "fire", enable: true, wuxieable: true,
    selectTarget: -1, reverseOrder: true,                                  // AOE + 逆序
    filterTarget(card, player, target) { return target !== player; },      // 其他角色
    content() {
        "step 0";
        if (target.countCards("h")) target.chooseToDiscard("机关火筒：弃置一张手牌或受到1点火焰伤害");
        else { target.damage("fire"); event.parent.preResult = true; }     // 无手牌直接火伤
        // …contentAfter：本次结算无实伤 → 使用者摸 1
    },
},
```

## 2 验收结果（`nc08_upstream_swd_jiguantong.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 有手牌 → `chooseToDiscard` | **分支 A**：AOE 三名目标**各弃一张** → 询问 `swd_jiguantong_discard` 出现、**hp 4 → 4（免伤）**、使用者手牌 **1 → 1**（打出本体 −1、**contentAfter 摸 1**） | 通过 |
| 2 | 无手牌 → `damage("fire")` | **分支 B**：目标**无手牌** → **无弃牌询问**（`询问=[]`）、**hp 4 → 3（火伤）**、使用者手牌 **1 → 0**（**有实伤 → 不摸牌**） | 通过 |
| 1 | `reverseOrder: true` | **差异登记**：上游**逆序**结算 ↔ 原生**座位序**（`play_card.rs:997` 注释自述「结算序=座位序登记差异」） | 通过（**差异登记**） |

**原生实现引用**：`content/host/step/swd_card_effects.rs:645`（逐目标两分支 + targets 空后按
日志实伤判定 `!damaged → draw_cards(owner,1)`）。

## 3 夹具要点（**AOE 门槛**）

该卡为 AOE，`engine/play_card.rs:997` 要求 `targets` **恰为**「自当前回合角色起的座位序中
**全部其他存活角色**」→ 夹具须传 **`vec![1,2,3]`**（传单个目标会 `Rejected(IllegalTargets)`）。
另：分支 A 须让**全部目标**都有手牌，否则未弃牌者受火伤 → `contentAfter` 判定为「有实伤」→ 使用者**不**摸牌。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（结算顺序）扩展到其它 AOE 卡（标准面 `wanjian`/`nanman` 等）核对是否同族
- [ ] `wuxieable` 无懈面；`cardnature: fire` 属性面
- [ ] 样本 69+：其余来源/卡牌（**swd 90 卡**仍有大量未采样）
