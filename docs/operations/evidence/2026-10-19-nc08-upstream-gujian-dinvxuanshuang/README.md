# NC08 第⑦层样本 96：**来源 U05（古剑）** 基本牌 `dinvxuanshuang`（帝女玄霜）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（目标集合范围，登记）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`（`dinvxuanshuang`）：

```js
dinvxuanshuang: {
    type: "basic", savable: true, selectTarget: -1,     // 可作濒死救援牌；**全部角色**
    content() {
        "step 0"; target.recover();                     // 回复 1 点体力
        "step 1";
        if (target.isIn()) target.chooseToDiscard([1, Infinity], "he", "弃置任意张牌并摸等量的牌");
        "step 2";
        if (result.bool) target.draw(result.cards.length);
    },
},
```

## 2 验收结果（`nc08_upstream_gujian_dinvxuanshuang.rs`）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动（使用者与 1 号**均受伤**时） | `Accepted` | 可发动 ✅ |
| **使用者**回复 | hp **3 → 4** | ✅ 符合上游 |
| **其他角色**回复（`selectTarget: -1`） | 1 号 hp **3 → 3** | ✗ **未回复** |

⇒ 原生**只回复使用者**，而上游 `selectTarget: -1` 为**全部角色**。

## 3 差异归类

**「目标集合范围」族第 2 例**（第 1 例为 `jieyan` 隙焰，样本 76）：

| 例 | 上游 | 原生 | 方向 |
|---|---|---|---|
| `jieyan`（隙焰） | `filterTarget: true` + `selectTarget: -1`（含使用者） | 排除使用者 | 少打 |
| **`dinvxuanshuang`（帝女玄霜）** | `selectTarget: -1`（**全部角色**） | **只含使用者** | 少回 |

⇒ 两例**方向不同**，但同属「**目标集合与上游不一致**」。

## 4 夹具要点

- 该卡为 `savable` + `selectTarget: -1` → 引擎按「**无显式目标**」形态受理（`vec![]`）；
  传 `vec![1]` 会 `Rejected(IllegalTargets)`；
- 使用者**满血**时会 `Rejected(HpFull)`（引擎按「自己回复」校验）→ 夹具须让**使用者受伤**。

## 5 未完成 / 待办

- [ ] **目标集合**修复（第 39 项）；`savable`（濒死救援）路径
- [ ] 弃 N 摸 N 的数值面；`ai` 策略面
- [ ] 古剑包其余卡牌
