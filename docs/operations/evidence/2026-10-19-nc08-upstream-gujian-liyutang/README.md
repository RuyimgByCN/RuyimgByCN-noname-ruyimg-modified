# NC08 第⑦层样本 89：**来源 U05（古剑）** 食物牌 `liyutang`（鲤鱼汤）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过**（门槛）+ **1 项未验收**（**用牌效果缺失**；子技效果**已实现**）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`liyutang`）：

```js
liyutang: {
    type: "food", enable: true,
    filterTarget(card, player, target) { return !target.hasSkill("liyutang"); },   // 目标未持该技
    content() {
        target.$gain2(cards);                        // 目标获得此牌
        target.storage.liyutang = card;
        target.storage.liyutang_markcount = 2;       // 2 个计数
        target.addSkill("liyutang");
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_liyutang.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `!target.hasSkill("liyutang")` | **门槛通过**：用 `debug_grant(1, "gj_food_liyutang")` 预置子技后，再对 1 号使用 → **`Rejected(IllegalTargets)`** | 通过 |
| 2 | `target.$gain2(cards)` + `storage` + `addSkill` | **未验收（用牌效果缺失）**：实测发动 `Accepted`，但**目标手牌 0 → 0**、**使用者手牌为空**、**该牌进入弃牌堆**（`弃堆含此牌=true`）⇒ 只完成用牌与弃置。**注**：其**子技效果已实现**（`step/gujian.rs:318`：阶段开始体力最少 → +1 护甲 + `food_tick`）→ 缺的是**用牌那一段** | 登记（第 37 项） |

**原生实现引用**：`vocabulary/host/keys.rs:347`（`HostBehavior::GujianFoodLiyutang => "host:gujian_food_liyutang"`）；
子技在**目录中的名字**为 **`gj_food_liyutang`**（`vocabulary/host/constants.rs:52`）。

## 3 夹具要点

- 包内**只有一张**鲤鱼汤 → 测门槛须用 **`debug_grant`** 预置子技（**目录名 `gj_food_liyutang`**，
  不是 `liyutang`——后者会 `unknown skill` panic）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed / 1 ignored** | [behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **实现获得牌 + storage 计数 + addSkill**（第 37 项）
- [ ] **计数的逐轮结算**（子技效果，需跨回合驱动）
- [ ] 古剑包其余卡牌
