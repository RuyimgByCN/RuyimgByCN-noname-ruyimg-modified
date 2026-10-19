# NC08 第⑦层样本 145：**来源 U05（运筹）** 锦囊 `chenhuodajie`（趁火打劫）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（含行为面）** + **族内差异确认**（「确定性首张」）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js`（`chenhuodajie`）：

```js
chenhuodajie: {
    type: "trick", filterTarget: true, global: "g_chenhuodajie",
    content() {
        if (target.countCards("he")) player.gainPlayerCard("he", target, true);   // **使用者选择**并获得
    },
},
```

## 2 验收结果（`nc08_upstream_yunchou_chenhuodajie.rs`）

| # | 判据 | 实测 | 结果 |
|---|---|---|---|
| 2 | **目标失去一张 `he` 牌** | 1 号 `he` **2 → 1** | 通过 |
| 2 | **使用者获得该牌** | 0 号手牌 **1 → 1**（打出 −1、获得 1） | 通过 |
| — | **选择权** | **`询问=[]`** ⇒ **自动取手牌首张**（原手牌已失去、装备保留） | **族内差异确认** |

**原生实现引用**：`catalog.rs:92`（注释：「趁火打劫（G1 运筹帷幄，锦囊）：获得目标一张手牌
（**登记差异**：…）」）。

## 3 差异归类

属既有「**确定性首张**」族（上游为**使用者选择** `gainPlayerCard(..., true)`，原生**自动取首张**）；
同类已见：盾牌格挡、照明弹、五行生克「被克」支等 ⇒ **非新项**。

## 4 未完成 / 待办

- [ ] **`global` 响应窗口**用法面（`g_chenhuodajie`）
- [ ] `ai` 面
