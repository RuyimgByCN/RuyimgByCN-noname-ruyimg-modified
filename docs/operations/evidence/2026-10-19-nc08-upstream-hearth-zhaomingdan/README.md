# NC08 第⑦层样本 **120**：**来源 U07（hearth）** 锦囊 `zhaomingdan`（照明弹）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）** + **1 项差异登记**（「确定性首张」族，**第 42 项**）
- **里程碑**：本样本为 **NC08 第 120 个样本**

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`zhaomingdan`）：

```js
zhaomingdan: {
    type: "trick", enable: true,
    filterTarget(card, player, target) {
        return player !== target && target.countCards("hej") > 0; },
    content() {
        "step 0"; if (target.countCards("hej")) {
            var next = player.discardPlayerCard("hej", target, true);   // **使用者选择**弃哪张
            next.visible = true; next.delay = false; }
        "step 2"; target.draw(false); target.$draw();                   // 目标摸 1
        "step 3"; player.draw();                                        // 使用者摸 1
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_zhaomingdan.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **弃目标 1 张 `hej`** | **无询问**；**目标手牌首张被弃**（`原手牌仍在=false`）；**装备未被弃** | 通过（净效果） |
| 2 | **目标摸 1 + 使用者摸 1** | 1 号手牌 **1 → 1**（弃 1 摸 1）；0 号手牌 **1 → 1**（打出 −1、摸 1） | 通过 |
| — | **差异登记** | 上游 `discardPlayerCard("hej", target, true)` 为**使用者选择**；原生为**自动取首张（手牌优先）** ⇒ **无询问** → **「确定性首张」族**（**第 42 项**） | 登记 |

**原生实现引用**：`kernel/intrinsic.rs:1278`（`CardKind::Zhaomingdan` → `HostBehavior::ZhaomingdanCard`）。

## 3 未完成 / 待办

- [ ] **弃牌选择权**（第 42 项；与既有「确定性首张」族同源）
- [ ] `visible` / `delay` 的**表现面**
- [ ] **判定区（`j`）为唯一来源**时的分支
