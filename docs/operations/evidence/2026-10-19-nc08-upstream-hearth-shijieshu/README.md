# NC08 第⑦层样本 124：**来源 U07（hearth）** 锦囊 `shijieshu`（视界术）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（装备面）** + **2 项差异登记**（**目标集合** + **末尾弃 1 缺失**）
- **意义**：**hearth 包已转换卡采样完成**（12 张中已采 8；余 3 张为 `jihuocard` 之外的 视界术已采）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`shijieshu`）：

```js
shijieshu: {
    type: "trick", enable: true,
    filterTarget(card, player, target) { return !target.isMin(); },   // 体力不是最少的角色
    content() {
        "step 0"; // 从牌堆找 2 张**不同 sub-type** 的装备牌
        "step 1"; if (event.cards.length) { target.equip(event.cards.shift()); … event.redo(); }
        "step 3"; if (target.countCards("he")) target.chooseToDiscard("he", true);   // **末尾弃 1**
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_shijieshu.rs`）

| # | 观测点 | 实测 | 判读 |
|---|---|---|---|
| 2 | **获得并装备**牌堆装备 | 使用者装备 **0 → 2** | ✅ 通过 |
| 2 | **末尾弃 1** | **`弃牌=false`**；`he` 总数 **1 → 2**（= +2 装备 −1 打出本体）⇒ **未发生弃牌** | ✗ **差异（第 45 项）** |
| 1 | **目标集合** | 传显式目标 → **`Rejected(IllegalTargets)`**；原生 `play_card.rs:952` 注释「**世界树：自-target**」 | ✗ **差异（第 45 项）** |

## 3 差异归类

**「目标集合范围」族第 3 例**（前两例：`jieyan` 隙焰、`dinvxuanshuang` 帝女玄霜）：

| 例 | 上游 | 原生 |
|---|---|---|
| `jieyan` | 含使用者（`selectTarget: -1`） | **排除**使用者 |
| `dinvxuanshuang` | 全部角色 | **只含**使用者 |
| **`shijieshu`** | **其他角色**（`!target.isMin()`） | **只含**使用者（**自-target**） |

## 4 未完成 / 待办

- [ ] **目标集合**修复（第 45 项）
- [ ] **末尾弃 1** 的实现（第 45 项）
- [ ] **sub-type 互不相同**这一约束的核对；装备来自**牌堆**的入口面
