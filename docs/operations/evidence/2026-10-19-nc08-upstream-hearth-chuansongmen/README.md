# NC08 第⑦层样本 119：**来源 U07（hearth）** 锦囊 `chuansongmen`（传送门）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）**，**未发现差异**（并行为面确认基准登记差异）
- **意义**：**hearth 包已采样卡覆盖完成**（6 张：盾牌格挡/治疗波/元素毁灭/坦蛇至人/星界沟通/传送门）

## 1 上游源码（归档快照，逐字引用要点）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`（`chuansongmen`）：

```js
chuansongmen: {
    type: "trick", enable: true, discard: false, toself: true,
    selectTarget: -1, filterTarget(card, player, target) { return target === player; },
    modTarget: true,
    content() {
        "step 0";
        var gained = get.cards()[0];                 // 牌堆顶一张
        target.gain(gained, "gain2");                // 获得之
        if (… lib.filter.filterCard(gained, target, …)) {
            var next = target.chooseToUse();         // 立即询问是否使用该牌
            next.filterCard = card => card === gained;
            next.prompt = "是否使用" + get.translation(gained) + "？";
        } else { … }
    },
},
```

## 2 验收结果（`nc08_upstream_hearth_chuansongmen.rs`）

| # | 要点 | 实测 | 结果 |
|---|---|---|---|
| 2 | **获得牌堆顶一张** | 手牌 **1 → 1**（打出 −1、获得 +1）；**牌堆 123 → 122**；**原牌堆顶（`Some(25)`）确在手中** | 通过 |
| — | **登记差异的行为面确认** | **`询问=[]`** ⇒ **无「是否使用该牌」询问**，与基准 golden 表第 55 行「**立即使用链暂缓**」一致 | 确认 |

**原生实现引用**：`kernel/intrinsic.rs:444/476`（`CardKind::Chuansongmen`）。

## 3 未完成 / 待办

- [ ] `modTarget` 的影响面
- [ ] `discard: false`（**本体去向**）的完整核对
- [ ] **立即使用链**的后续实现（基准已登记为「暂缓」）
