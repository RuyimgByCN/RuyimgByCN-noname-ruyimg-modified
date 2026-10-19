# NC08 第⑦层样本 126：**来源 U05（运筹）** 延时锦囊 `huoshan`（火山）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项未验收**（**目标形态与上游相反 + 效果未实现**；**同日更正**：初版「完全不可用」不准确）
- **意义**：三维核对（台账第 126 行）中**唯一**「无实现且无登记」的卡，本样本给出**行为面基线**

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js`（`huoshan`）：

```js
huoshan: {
    fullskin: true, type: "delay", cardcolor: "red", cardnature: "fire", toself: true,
    modTarget(card, player, target) { return lib.filter.judge(card, player, target); },
    enable(card, player) { return player.canAddJudge(card); },
    filterTarget(card, player, target) {
        return lib.filter.judge(card, player, target) && player === target; },
    selectTarget: [-1, -1],
    judge(card) {                       // 判定：红桃 2–9 → 通过（−6）
        if (get.suit(card) === "heart" && get.number(card) > 1 && get.number(card) < 10) return -6;
        return 1; },
    judge2(result) { if (result.bool === false) return true; return false; },   // 反转
    effect() {
        if (result.bool === false) {
            player.damage(2, "fire", "nosource");     // 自己受 2 点火伤（无来源）
            var players = game.filterPlayer(current =>
                get.distance(player, current) <= 1 && player !== current);
            players.sort(lib.sort.seat);              // 距离 ≤1 的其他角色（座位序）
            …
        }
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_huoshan.rs`，**登记基线**）

| 观测点 | 实测 | 判读 |
|---|---|---|
| 发动（`targets: vec![]`） | **`Rejected(IllegalTargets)`** | ✗ |
| 发动（`targets: vec![0]`，显式自己） | **`Rejected(IllegalTargets)`** | ✗ |
| 发动（**`targets: vec![1]`，其他角色**） | **`Accepted`** | ✗ **与上游 `toself` 相反** |
| 判定区 | **0 → 0**（未进入判定区） | ✗ |

⇒ **精度更正（同日）**：初版只试了两种形态（`[]` 与 `[0]`）便判定「**完全不可用**」——**不准确** ✗。
**第三种形态（其他角色 `[1]`）会被受理** ⇒ 真实情况是：
**只受理「其他角色」目标，拒绝「无目标/自己」** —— 与上游 **`toself: true`（只对自己）相反**；
且受理后**判定区 `0 → 0`**（**效果未实现**）。

## 3 处置（第 47 项）

- **实现维**：`crates/engine-native/src/` **0 命中**；
- **登记维**：`docs/**` **0 命中**；
- **行为面**：**完全不可用**（两形态均 `Rejected`）。

**修法**：按上游实现**延时锦囊**（进判定区、`judge` 红桃 2–9 通过、`judge2` 反转、
`effect` 自己 2 点火伤 + 距离 ≤1 其他角色按座位序结算），并补登记。

**验收**：本用例应能断言「发动被受理且进入判定区」，随后补判定分支样本。

## 4 未完成 / 待办

- [ ] **火山实现**（第 47 项）
- [ ] `effect` 尾段（相邻角色后续结算）与 `judge2` 反转的逐字确认
