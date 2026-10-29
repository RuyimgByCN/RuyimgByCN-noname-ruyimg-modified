# NC08 第⑦层样本 127：**来源 U08（阵法）** 阵法牌 `dizaizhen`（地载阵）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项通过（摸 1）** + **1 项差异登记**（**模式门槛缺失**，第 49 项）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js`（`dizaizhen`）：

```js
dizaizhen: {
    type: "zhenfa", recastable: true,
    enable() { return game.hasPlayer(current => current.isNotMajor()); },
    filterTarget(card, player, target) { return target.isNotMajor(); },   // 非主公
    selectTarget: -1,
    content() { target.draw(false); target.$draw(); },                    // 摸 1
    mode: ["guozhan"],                                                    // **仅国战模式**
},
```

## 2 验收结果（`nc08_upstream_zhenfa_dizaizhen.rs`）

| # | 观测点 | 实测 | 判读 |
|---|---|---|---|
| 3 | **摸 1** | 1 号手牌 **1 → 2** | ✅ 通过 |
| 1 | **模式门槛**（`mode: ["guozhan"]`） | **身份局（4 人）下 `vec![]` → `Accepted`** ⇒ **仍可发动** | ✗ **差异（第 49 项）** |
| — | 目标形态 | `vec![1]` → `Rejected`；**`vec![]` → `Accepted`**（`selectTarget: -1` 按「无显式目标」受理） | 记录 |

**原生实现引用**：`kernel/intrinsic.rs:179`（`CardKind::Dizaizhen` → `DizaizhenMachine`）。

## 3 差异归类

**「门槛缺失」族第 2 例**（第 1 例：乾坤镖 `he` 门槛，修复清单第 35 项）：

| 例 | 上游门槛 | 原生 |
|---|---|---|
| `qiankunbiao` | 目标 `he` 有牌 | **未校验** |
| **`dizaizhen`** | **`mode: ["guozhan"]`（仅国战）** | **身份局仍可发动** |

## 4 未完成 / 待办

- [ ] **模式门槛**实现（第 49 项；并核对**其余 `mode: ["guozhan"]` 卡**）
- [ ] `recastable`（重铸）面；国战模式下 `isNotMajor` 的完整语义
