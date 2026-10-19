# NC08 第⑦层样本 104：**来源 U05（古剑）** 食物牌 `qingtuan`（青团）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（首杀摸 1 + **门控已由样本 105 补验闭环**）
- **作用**：**食物族「非归属效果」抽样**（归属面已由样本 94 全量普查）

## 1 上游源码（归档快照，逐字引用）

`.../gujian/card.js`（`qingtuan`）+ `.../gujian/skill.js:225`（子技 `qingtuan`）：

```js
// card.js
qingtuan: { type: "food", enable: true,
    filterTarget(card, player, target) { return !target.hasSkill("qingtuan"); },
    content() { target.$gain2(cards); target.storage.qingtuan_markcount = 2; target.addSkill("qingtuan"); } }
// skill.js:225（子技）
qingtuan: { … 提示「你在回合内使用首张杀时摸一张牌（剩余 N 回合）」…
            player.storage.qingtuan_markcount--; }
```

## 2 验收结果（`nc08_upstream_gujian_qingtuan.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 子技「**回合内使用首张杀时摸一张牌**」 | **行为面**：吃完青团（子技已授予 `true`）后打**首张【杀】**→ **手牌不变**（打出杀 −1、**摸 1** ⇒ 净 0） | 通过 |
| 3 | `once_per_turn` 门控 | **已闭环（由样本 105 补验）**：同回合第二张【杀】在本样本被 `ShaLimit` 拒绝 ⇒ 改用同族**元宝肉**（额外一张杀）解开阻塞 → **第二张杀被允许且不再摸牌**（手牌 2 → 1）⇒ **门控成立** | 通过（跨样本补验） |

**原生实现引用**：`content/host/step/gujian.rs:247`（`GujianFoodQingtuan`：注释「回合内使用首张杀
（once_per_turn 门控）：摸 1」+ `food_tick`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed**（含 1 条 NOTE） | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [x] **门控已验证**（样本 105 用元宝肉补验：第二杀被允许且不再摸）
- [ ] `markcount` 的**逐回合递减与到期**
- [ ] `filterTarget` 门槛
- [ ] 食物族其余卡的非归属效果（元宝肉/骨董羹/茉莉茶/虾饺/蜜汁藕/鲈鱼羹/酉羹/昙花冻）
