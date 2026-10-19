# NC08 第⑦层样本 128：**来源 U06（swd）** 灵件牌 `hslingjian_zhongxinghujia`（重型护甲）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项差异登记**（**门槛口径不一致**，第 50 项）
- **意义**：全包受理普查的**重点待核项**（装备类被拒不合常理）**已定性**：**非缺陷、非缺失**，
  而是**门槛口径与上游不同**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js`：

```js
hslingjian_zhongxinghujia: {
    type: "hslingjian", fullimage: true, vanish: true, enable: true,
    derivation: true, derivationpack: "swd",
    filterTarget(card, player, target) { return !target.isMin(); },   // **体力不是最少**
    content() { "step 0"; … 生成一张 subtype==="equip2"（防具）并 target.equip(card) … },
},
```

## 2 原生实现（逐字引用）

`engine/play_card.rs:1645`（`CardKind::Hslingjian` 的**逐卡目标规则**）：

```rust
} else if cname.contains("zhongxinghujia") {
    targets.len() == 1
        && core.player(targets[0]).alive
        && core.player(targets[0]).hand.len() as i32 > core.player(targets[0]).hp   // **手牌 > 体力**
}
```

## 3 验收结果（`nc08_upstream_swd_zhongxinghujia.rs`）

| 局面 | 上游应 | 实测 | 判读 |
|---|---|---|---|
| ① 目标**手牌 3 > 体力 2** | 受理（体力非最少） | **`Accepted`** | 与**原生口径**一致 |
| ② 目标**手牌 1 ≤ 体力 4**（**体力非最少**） | **受理** | **`Rejected(IllegalTargets)`** | ✗ **差异** |

⇒ 原生门槛为「**手牌多于体力**」，上游为「**体力不是最少**」——**两者不等价**（局面②即为反例）。

## 4 未完成 / 待办

- [ ] **门槛口径对齐**（第 50 项）
- [ ] `vanish` / `derivation`（衍生与消失）面
- [ ] 效果面（生成防具并令目标装备）
